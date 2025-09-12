mod buffer_list;
use buffer_list::{Buffer, BufferId, BufferList};

mod pattern;
use pattern::{Pattern, Target};

mod rank;
use rank::{Item as RankingItem, rank};

use nvim_router::NeovimWriter;
use nvim_router::RpcArgs;
use nvim_router::nvim_rs::{Neovim, Value};

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
struct States {
    current_tab: BufferList,
    other_tabs: BufferList,
}

impl States {
    fn update(&mut self, current_tab: Vec<Value>, other_tabs: Vec<Value>, cwd: &str) {
        let home_dir = std::env::home_dir();
        let home_dir = home_dir
            .as_ref()
            .and_then(|path| path.to_str())
            .unwrap_or_default();

        self.current_tab = to_list(current_tab, cwd, home_dir);
        self.other_tabs = to_list(other_tabs, cwd, home_dir);
    }

    fn ranking(&self, input: &Pattern) -> Value {
        let current_tab = ranking_to_args(rank(&self.current_tab, input));
        let other_tabs = ranking_to_args(rank(&self.other_tabs, input));
        Value::Map(vec![
            (Value::from("current_tab"), current_tab),
            (Value::from("other_tabs"), other_tabs),
        ])
    }
}

fn to_list(buffers: Vec<Value>, cwd: &str, home_dir: &str) -> BufferList {
    buffers
        .into_iter()
        .filter_map(move |buf_item| {
            if let Value::Array(buf_item) = buf_item
                && let Some(id) = buf_item.first()
                && let Some(path) = buf_item.get(1)
                && let Some(path) = path.as_str()
                && let Some(metadata) = buf_item.get(2)
            {
                let path = if let Some(rest) = path.strip_prefix(cwd) {
                    let mut path = Target::with_capacity(rest.len() + 1);
                    path.push('.');
                    path.push_str(rest);
                    path
                } else if let Some(rest) = path.strip_prefix(home_dir) {
                    let mut path = Target::with_capacity(rest.len() + 1);
                    path.push('~');
                    path.push_str(rest);
                    path
                } else {
                    Target::from_str(path)
                };

                Some(Buffer {
                    id: BufferId::from_id(id),
                    file: path,
                    metadata: metadata.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn ranking_to_args<'a>(ranking: impl IntoIterator<Item = RankingItem<'a>>) -> Value {
    let values = ranking
        .into_iter()
        .map(|item| {
            let matched = item
                .matched
                .into_iter()
                .map(|range| {
                    Value::Map(vec![
                        (Value::from("start_idx"), Value::from(range.start)),
                        (Value::from("end_idx"), Value::from(range.end)),
                    ])
                })
                .collect();
            Value::Array(vec![
                Value::from(item.buf_id),
                Value::from(item.content.display_name()),
                item.metadata,
                Value::Array(matched),
            ])
        })
        .collect();
    Value::Array(values)
}

#[derive(Debug, Clone, Default)]
pub struct NeovimHandler {
    states: Arc<Mutex<States>>,
}

impl<W: NeovimWriter> nvim_router::NeovimHandler<W> for NeovimHandler {
    fn new() -> Self {
        Self::default()
    }

    async fn handle_request(
        &self,
        name: &str,
        mut args: RpcArgs,
        _neovim: Neovim<W>,
    ) -> Result<Value, Value> {
        if name == "rank" {
            let Some(input) = args.next_string() else {
                return Ok(Value::Nil);
            };
            let input = Pattern::from_string(input);

            let lock = self.states.lock().await;
            let ret = lock.ranking(&input);
            Ok(ret)
        } else {
            Ok(Value::Nil)
        }
    }

    async fn handle_notify(&self, name: &str, mut args: RpcArgs, _neovim: Neovim<W>) {
        if name == "update_buffers" {
            let Some(current) = args.next_array() else {
                return;
            };
            let Some(other) = args.next_array() else {
                return;
            };
            let Some(cwd) = args.next_string() else {
                return;
            };

            let mut lock = self.states.lock().await;
            lock.update(current, other, &cwd);
        }
    }
}
