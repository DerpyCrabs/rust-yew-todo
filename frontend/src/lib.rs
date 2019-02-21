#![recursion_limit = "128"]
#![allow(unused_variables)]
#![feature(nll)]
#![feature(vec_remove_item)]

#[macro_use]
extern crate yew;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate failure;
#[macro_use]
extern crate stdweb;

mod entry;

use entry::{Entry, EntryId, Folder, Task};
use failure::Error;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::storage::{Area, StorageService};

const KEY: &'static str = "derpycrabs.todolist.self";

pub struct Model {
    state: State,
    storage: StorageService,
    fetch: FetchService,
    fetch_handler: Option<FetchTask>,
    emitter: Callback<State>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    root: Entry,
    view: usize,
    id: EntryId,
    input: String,
    add_task: usize,
    add_folder: usize,
    editing: Option<usize>,
}

#[derive(Clone)]
pub enum Msg {
    GetState,
    PostState,
    GotState(State),

    Go(usize),
    GoBack,

    Delete(usize),
    AddTask,
    AddFolder,
    MoveUp(usize),
    MoveDown(usize),

    Toggle(usize),
    BeginInput(usize),
    UpdateInput(String),
    FinishInput(usize),

    Nope,
}

impl State {
    fn move_entry(&mut self, folder_id: usize, item_id: usize, position: usize) -> bool {
        let entry_clone;
        if let Some(entry) = self.root.find_mut(item_id) {
            entry_clone = entry.clone()
        } else {
            return false;
        }
        if let Some(Entry::Folder(folder)) = self.root.find_mut(folder_id) {
            folder.entries.remove_item(&entry_clone);
            folder.entries.insert(position, entry_clone);
        }
        true
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut storage = StorageService::new(Area::Local);
        let fetch = FetchService::new();
        let emitter = link.send_back(Msg::GotState);

        let state = if let Json(Ok(restored_state)) = storage.restore(KEY) {
            restored_state
        } else {
            let mut id = EntryId::new();
            let root_id = id.get();
            let add_task = id.get();
            let add_folder = id.get();

            let root = Entry::Folder(Folder {
                name: "Root folder".to_string(),
                entries: Vec::new(),
                parent: None,
                id: root_id,
            });
            State {
                root,
                view: root_id,
                id,
                add_task,
                add_folder,
                input: "".to_string(),
                editing: None,
            }
        };

        Model {
            state,
            storage,
            fetch,
            fetch_handler: None,
            emitter,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetState => {
                let emitter = self.emitter.clone();
                let url = format!("/tasks");
                let request = Request::get(url.as_str()).body(Nothing).unwrap();
                let handle = move |response: Response<Json<Result<State, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    if meta.status.is_success() {
                        emitter.emit(data.expect("error response from GET /tasks"));
                    }
                };
                self.fetch_handler = Some(self.fetch.fetch(request, handle.into()));
            }
            Msg::PostState => {
                let url = format!("/tasks");
                let handler = move |_response: Response<Json<Result<(), Error>>>| {};
                let mut state = self.state.clone();
                state.editing = None;
                state.view = state.root.id();
                let as_json = json!{state};
                let request = Request::post(url.as_str())
                    .header("Content-Type", "application/json")
                    .body(Ok(as_json.to_string()))
                    .unwrap();
                self.fetch_handler = Some(self.fetch.fetch(request, handler.into()));
            }
            Msg::GotState(state) => self.state = state,

            Msg::Go(id) => self.state.view = id,
            Msg::GoBack => match self.state.root.find(self.state.view).unwrap() {
                Entry::Folder(Folder { parent, .. }) => match parent {
                    Some(id) => self.state.view = *id,
                    None => return false,
                },
                _ => return false,
            },

            Msg::Delete(id) => match self.state.root {
                Entry::Folder(ref mut folder) => folder.delete(id),
                _ => return false,
            },
            Msg::AddTask => {
                self.state.editing = None;
                if !self.state.input.is_empty() {
                    if let Some(Entry::Folder(folder)) = self.state.root.find_mut(self.state.view) {
                        (*folder).entries.push(Entry::Task(Task {
                            name: self.state.input.drain(..).collect(),
                            done: false,
                            id: self.state.id.get(),
                        }));
                    }
                }
                js! {
                    document.querySelector(".footer > .edit").blur();
                }
            }
            Msg::AddFolder => {
                self.state.editing = None;
                if !self.state.input.is_empty() {
                    if let Some(Entry::Folder(folder)) = self.state.root.find_mut(self.state.view) {
                        (*folder).entries.push(Entry::Folder(Folder {
                            name: self.state.input.drain(..).collect(),
                            id: self.state.id.get(),
                            parent: Some(folder.id),
                            entries: Vec::new(),
                        }));
                    }
                }
                js! {
                    document.querySelector(".footer > .edit").blur();
                }
            }
            Msg::MoveUp(id) => {
                if let Some(Entry::Folder(folder)) = self.state.root.find(self.state.view) {
                    let position: usize =
                        match folder.entries.iter().position(|entry| entry.id() == id) {
                            Some(0) => folder.entries.len() - 1,
                            Some(p) => p - 1,
                            None => return false,
                        };
                    self.state.move_entry(self.state.view, id, position);
                }
            }
            Msg::MoveDown(id) => {
                if let Some(Entry::Folder(folder)) = self.state.root.find(self.state.view) {
                    let position: usize =
                        match folder.entries.iter().position(|entry| entry.id() == id) {
                            Some(p) if p == folder.entries.len() - 1 => 0,
                            Some(p) => p + 1,
                            None => return false,
                        };
                    self.state.move_entry(self.state.view, id, position);
                }
            }

            Msg::Toggle(id) => match self.state.root.find_mut(id).unwrap() {
                Entry::Folder(_) => return false,
                Entry::Task(task) => task.done ^= true,
            },
            Msg::BeginInput(id) => self.state.editing = Some(id),
            Msg::UpdateInput(val) => self.state.input = val,
            Msg::FinishInput(id) => {
                if self.state.editing.is_none() {
                    return false;
                };
                self.state.editing = None;
                if !self.state.input.is_empty() {
                    match self.state.root.find_mut(id).unwrap() {
                        Entry::Folder(folder) => {
                            folder.name = self.state.input.drain(..).collect();
                        }
                        Entry::Task(task) => {
                            task.name = self.state.input.drain(..).collect();
                        }
                    }
                }
            }

            Msg::Nope => (),
        }
        let mut state = self.state.clone();
        state.editing = None;
        state.view = state.root.id();
        self.storage.store(KEY, Json(&state));
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let state = &self.state;
        let add_task_need_input = match state.editing {
            Some(fid) if fid == state.add_task => true,
            _ => false,
        };
        let add_folder_need_input = match state.editing {
            Some(fid) if fid == state.add_folder => true,
            _ => false,
        };
        let add_task = input_view(
            "Task name".to_string(),
            Msg::BeginInput(state.add_task),
            Msg::AddTask,
            add_task_need_input,
            true,
        );
        let add_folder = input_view(
            "Folder name".to_string(),
            Msg::BeginInput(state.add_folder),
            Msg::AddFolder,
            add_folder_need_input,
            true,
        );
        html! {
            <div class="app",>
                <div class="header",>
                    <button class="header-button", onclick=|_| Msg::GetState,>{"Get State"}</button>
                    <button class="header-button", onclick=|_| Msg::PostState,>{"Post State"}</button>
                </div>
                { self.state.root.find(self.state.view).unwrap().view(&self.state) }
                <div class="footer",>
                    { add_task }
                    { add_folder }
                </div>
            </div>
        }
    }
}

fn input_view(
    initial_value: String,
    begin: Msg,
    finish: Msg,
    editing: bool,
    placeholder: bool,
) -> Html<Model> {
    if editing {
        let finish_clone = finish.clone();
        if placeholder {
            html! {
            <input class="edit", size=1, type="text", placeholder={initial_value},
                oninput=|e| Msg::UpdateInput(e.value),
                onkeypress=|e| if e.key() == "Enter" { finish.clone() } else { Msg::Nope },
                onblur=|_| finish_clone.clone(), autofocus="autofocus",/>
            }
        } else {
            html! {
            <input class="edit", size=1, type="text", value={initial_value},
                oninput=|e| Msg::UpdateInput(e.value),
                onkeypress=|e| if e.key() == "Enter" { finish.clone() } else { Msg::Nope },
                onblur=|_| finish_clone.clone(), autofocus="autofocus",/>
            }
        }
    } else {
        html! {
            <input class="edit", size=1, value={initial_value},
                type="text",
                onclick=|_| begin.clone(), readonly="readonly",/>
        }
    }
}
