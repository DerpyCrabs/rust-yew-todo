use super::{input_view, Model, Msg, State};
use yew::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct EntryId {
    pub latest_id: usize,
}

impl EntryId {
    pub fn new() -> EntryId {
        EntryId { latest_id: 0 }
    }

    pub fn get(&mut self) -> usize {
        self.latest_id = self.latest_id + 1;
        self.latest_id
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Entry {
    Folder(Folder),
    Task(Task),
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Folder {
    pub name: String,
    pub entries: Vec<Entry>,
    pub parent: Option<usize>,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Task {
    pub name: String,
    pub done: bool,
    pub id: usize,
}

impl Task {
    pub fn view(&self, state: &State) -> Html<Model> {
        let editing = match state.editing {
            Some(fid) if fid == self.id => true,
            _ => false,
        };
        let id = self.id;
        let task_name = input_view(
            self.name.clone(),
            Msg::BeginInput(self.id),
            Msg::FinishInput(self.id),
            editing,
            false,
        );
        html! {
            <li class="task",>
                <input class="task-toggle", type="checkbox",
                    checked=self.done.clone(), onchange=|_| Msg::Toggle(id), />
                { task_name }
                <button class="up", onclick=|_| Msg::MoveUp(id), />
                <button class="down", onclick=|_| Msg::MoveDown(id), />
                <button class="delete", onclick=|_| Msg::Delete(id), />
            </li>
        }
    }
}

impl Entry {
    pub fn id(&self) -> usize {
        match self {
            Entry::Folder(Folder { id, .. }) => *id,
            Entry::Task(Task { id, .. }) => *id,
        }
    }

    pub fn view(&self, state: &State) -> Html<Model> {
        match self {
            Entry::Task(task) => task.view(state),
            Entry::Folder(folder) => folder.view(state),
        }
    }

    pub fn find_mut(&mut self, id: usize) -> Option<&mut Entry> {
        match self {
            Entry::Folder(Folder { id: fid, .. }) if *fid == id => Some(self),
            Entry::Task(Task { id: fid, .. }) if *fid == id => Some(self),
            Entry::Folder(Folder { entries, .. }) => entries
                .iter_mut()
                .map(|f| f.find_mut(id))
                .find(|f| f.is_some())
                .map(|f| f.unwrap()),
            _ => None,
        }
    }

    pub fn find(&self, id: usize) -> Option<&Entry> {
        match self {
            Entry::Folder(Folder { id: fid, .. }) if *fid == id => Some(self),
            Entry::Task(Task { id: fid, .. }) if *fid == id => Some(self),
            Entry::Folder(Folder { entries, .. }) => entries
                .iter()
                .map(|f| f.find(id))
                .find(|f| f.is_some())
                .map(|f| f.unwrap()),
            _ => None,
        }
    }
}

impl Folder {
    pub fn delete(&mut self, id: usize) {
        for entry in self.entries.iter_mut() {
            if let Entry::Folder(folder) = entry {
                folder.delete(id);
            }
        }
        self.entries.retain(|e| match e {
            Entry::Folder(Folder { id: fid, .. }) if *fid == id => false,
            Entry::Task(Task { id: fid, .. }) if *fid == id => false,
            _ => true,
        });
    }

    pub fn view(&self, state: &State) -> Html<Model> {
        let id = self.id;
        let need_input = match state.editing {
            Some(fid) if fid == id => true,
            _ => false,
        };
        let folder_name = input_view(
            self.name.clone(),
            Msg::BeginInput(self.id),
            Msg::FinishInput(self.id),
            need_input,
            false,
        );

        let elements = self.entries.iter().map(|entry| match entry {
            Entry::Task(task) => task.view(state),
            Entry::Folder(folder) => {
                let id = folder.id;
                html! {
                    <li class="folder",>
                        <div class="folder-icon",></div>
                        <input type="text", onclick=|_| Msg::Go(id), class="edit", size=1, value={folder.name.as_str()}, readonly="readonly",></input>
                        <button class="up", onclick=|_| Msg::MoveUp(id), />
                        <button class="down", onclick=|_| Msg::MoveDown(id), />
                        <button class="delete", onclick=|_| Msg::Delete(id), />
                    </li>
                }
            }
        });

        let back_button = if self.parent.is_some() {
            html! {<div class="folder-back", onclick=|_| Msg::GoBack,></div>}
        } else {
            html! {<div></div>}
        };
        html! {
            <div class="folder-view",>
                <div class="folder-name-container row", onclick=|_| Msg::BeginInput(id),>
                {back_button}{ folder_name }
                </div>
                <ul class="entries",>
                    {for elements}
                </ul>
            </div>
        }
    }
}
