use std::rc::Rc;

use quick_search_lib::{PluginId, SearchResult};

use crate::search_instance::SearchMetadata;

#[derive(Default)]
pub struct ResultHolder {
    results: Vec<ResultGroup>,
    cursor: usize,
}

impl ResultHolder {
    pub fn clear(&mut self) {
        self.results.clear();
        self.cursor = 0;
    }
    pub fn add_results(&mut self, results: Vec<SearchResult>, metadata: SearchMetadata) {
        let res_len = self.results.len();
        let this_name = metadata.raw_name.clone();
        self.results.push(ResultGroup { results, metadata: Rc::new(metadata) });
        self.results.sort_by(|a, b| b.metadata.priority.cmp(&a.metadata.priority));
        // if cursor was located after where the new result was added, add the length of the new results to the cursor
        let mut len_before = 0;
        for results in &self.results {
            if results.metadata.raw_name == this_name {
                break;
            }
            len_before += results.results.len();
        }
        if self.cursor > len_before {
            self.cursor += res_len;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.results.iter().all(|x| x.results.is_empty())
    }

    pub fn increment_cursor(&mut self) {
        self.cursor += 1;
        if self.cursor >= self.len() {
            self.cursor = 0;
        }
    }

    pub fn decrement_cursor(&mut self) {
        if self.cursor == 0 {
            self.cursor = self.len().saturating_sub(1);
        } else {
            self.cursor -= 1;
        }
    }

    pub fn clear_cursor(&mut self) {
        self.cursor = 0;
    }

    pub fn len(&self) -> usize {
        self.results.iter().map(|g| g.results.len()).sum()
    }

    pub fn jump_backward(&mut self, selected: bool) {
        // we want to set self.cursor to the last result of the previous group
        // if there is no previous group, set self.cursor to the last result of the last group

        if selected {
            let mut cursor = None;

            let mut done = false;

            let mut end_of_final_group = 0;

            for (i, (y, _)) in self.results.iter().flat_map(|g| g.results.iter().enumerate()).enumerate() {
                // i is the index in GLOBAL SPACE
                // the cursor is in GLOBAL SPACE
                // we need to detect the bounds of the previous group, so cursor will be the last time we crossed a group boundary
                // once we cross self.cursor for i, we can break

                if y == 0 && !done {
                    cursor = i.checked_sub(1);
                }
                if i == self.cursor {
                    done = true;
                }
                end_of_final_group = i;
            }

            self.cursor = cursor.unwrap_or(end_of_final_group);
        } else {
            // not selected, jump to final result of last group
            self.cursor = self.results.iter().flat_map(|g| g.results.iter()).count().saturating_sub(1);
        }
    }

    pub fn jump_forward(&mut self, selected: bool) {
        // we want to set self.cursor to the first result of the next group
        // if there is no next group, set self.cursor to the first result of the first group

        if selected {
            let mut cursor = None;
            let mut break_next = false;

            for (i, (y, _)) in self.results.iter().flat_map(|g| g.results.iter().enumerate()).enumerate() {
                // i is the index in GLOBAL SPACE
                // the cursor is in GLOBAL SPACE
                // we need to detect the bounds of the next group, so cursor will be the first time we cross a group boundary
                // once we cross self.cursor for i, we can break

                if y == 0 && break_next {
                    log::trace!("breaking at {}", i);
                    cursor = Some(i);
                    break;
                }
                if i == self.cursor {
                    break_next = true;
                }
            }

            self.cursor = cursor.unwrap_or(0);
        } else {
            // not selected, jump to first result of first group
            self.cursor = 0;
        }
    }

    pub fn get_from_cursor(&mut self) -> Option<(&SearchResult, PluginId)> {
        if self.results.is_empty() {
            return None;
        }

        let mut cursor = self.cursor;

        for group in self.results.iter_mut() {
            if cursor < group.results.len() {
                return group.results.get(cursor).map(|x| (x, group.metadata.id.clone()));
            } else {
                cursor -= group.results.len();
            }
        }

        None
    }

    pub fn iter_nice(&self, selected: bool) -> impl Iterator<Item = NiceIter> {
        // iterate over all groups
        // yield a NewSource every time the source changes
        // yield the first 3 results from each group UNLESS we are selected AND the cursor is in that group, then yield the 7 results centered around the cursor (from the self.cursor_range() function)

        // let mut vec = vec![];

        // for (i, (y, result)) in self.groups.iter().flat_map(|g| g.results.iter().enumerate()).enumerate() {
        //     if y == 0 {
        //         vec.push(NiceIter::NewSource(result.source.copy()));
        //     }
        //     if selected {
        //         // we have to handle the range checks
        //     } else {
        //         if y < 3 {
        //             vec.push(NiceIter::Result { result, cursor_on: false });
        //         }
        //     }
        // }

        // vec.iter()

        self.results
            .iter()
            .flat_map(|g| g.results.iter().map(|r| (r, g.metadata.clone())).enumerate())
            .enumerate()
            .flat_map(move |(i, (y, (result, group)))| {
                let res = if selected {
                    if self.cursor_range().contains(&i) {
                        Some(NiceIter::Result {
                            result,
                            cursor_on: { self.cursor == i },
                            // metadata: group.clone(),
                        })
                    } else {
                        None
                    }
                } else if y < 3 {
                    Some(NiceIter::Result {
                        result,
                        cursor_on: false,
                        // metadata: group.clone(),
                    })
                } else {
                    None
                };

                if y == 0 {
                    if let Some(res) = res {
                        vec![NiceIter::NewSource(group), res]
                    } else {
                        vec![NiceIter::NewSource(group)]
                    }
                } else if let Some(res) = res {
                    vec![res]
                } else {
                    vec![]
                }
            })
    }
    fn cursor_range(&self) -> std::ops::Range<usize> {
        let mut start = self.cursor.saturating_sub(2);
        let mut end = self.cursor.saturating_add(3);
        let mut range = start..end;
        let mut last_source = "".to_owned();
        for (x, (_y, g)) in self.results.iter().flat_map(|g| g.results.iter().map(|_| g.metadata.clone()).enumerate()).enumerate() {
            if range.contains(&x) && last_source != g.raw_name {
                last_source = g.raw_name.clone();
                if x <= self.cursor {
                    start = x;
                } else {
                    end = x;
                    break;
                }
            }
            range = start..end;
        }
        start..end
    }
}

pub struct ResultGroup {
    results: Vec<SearchResult>,
    metadata: Rc<SearchMetadata>,
}

pub enum NiceIter<'a> {
    NewSource(Rc<SearchMetadata>),
    Result {
        result: &'a SearchResult,
        cursor_on: bool,
        // metadata: Rc<SearchMetadata>,
    },
}
