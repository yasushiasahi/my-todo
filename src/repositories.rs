use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use thiserror::Error;

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateTodo {
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

type TodoDates = HashMap<i32, Todo>;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoDates>>,
}

impl TodoRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<TodoDates> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<TodoDates> {
        self.store.read().unwrap()
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_store_ref();

        let id = (store.len() + 1) as i32;
        let todo = Todo::new(id, payload.text);
        store.insert(id, todo.clone());

        todo
    }

    fn find(&self, id: i32) -> Option<Todo> {
        let store = self.read_store_ref();

        store.get(&id).cloned()
    }

    fn all(&self) -> Vec<Todo> {
        let store = self.read_store_ref();

        store.values().cloned().collect()
    }

    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref();

        let todo = store.get(&id).context(RepositoryError::NotFound(id))?;
        let todo = Todo {
            id,
            text: payload.text.unwrap_or(todo.text.clone()),
            completed: payload.completed.unwrap_or(todo.completed),
        };
        store.insert(id, todo.clone());

        Ok(todo)
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).context(RepositoryError::NotFound(id))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_crud_scenario() {
        let text = "todo  text";
        let id = 1;
        let expected = Todo::new(id, text.to_string());

        let reopsitory = TodoRepositoryForMemory::new();
        let todo = reopsitory.create(CreateTodo {
            text: text.to_string(),
        });
        assert_eq!(expected, todo);

        let todo = reopsitory.find(id).unwrap();
        assert_eq!(expected, todo);

        let todos = reopsitory.all();
        assert_eq!(vec![expected], todos);

        let text = "update todo text";
        let todo = reopsitory
            .update(
                id,
                UpdateTodo {
                    text: Some(text.to_string()),
                    completed: Some(true),
                },
            )
            .unwrap();
        assert_eq!(
            Todo {
                id,
                text: text.to_string(),
                completed: true
            },
            todo
        );

        let res = reopsitory.delete(id);
        assert!(res.is_ok());
    }
}
