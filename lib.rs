use colored::*;
use std::f32::consts::E;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter, Stdout, Write};
use std::path::Path;
use std::{env, process};

pub struct Todo {
    pub todo: Vec<String>,
    pub todo_path: String,
    pub todo_bak: String,
    pub no_backup: bool,
}

impl Todo {
    pub fn new() -> Result<Self, String> {
        let todo_path: String = match env::var("TODO_PATH") {
            Ok(t) => t,
            Err(_) => {
                let home = env::var("HOME").expect("Could not find $HOME");

                // look for a legacy TODO file path
                let legacy_todo = format!("{}/TODO", &home);
                match Path::new(&legacy_todo).exists() {
                    true => legacy_todo,
                    false => format!("{}/.todo", &home),
                }
            }
        };

        let todo_bak: String = match env::var("TODO_BAK_DIR") {
            Ok(t) => t,
            Err(_) => String::from("/tmp/todo.bak"),
        };

        let no_backup = env::var("TODO_NOBACKUP").is_ok();
        let todo_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&todo_path)
            .expect("Couldn't open the todofile");

        // Creates a new buf reader
        let mut buf_reader = BufReader::new(&todo_file);

        // Empty String read to be filled with TODOS
        let mut contents = String::new();

        // loads contents with data
        buf_reader.read_to_string(&mut contents).unwrap();

        // splits contents into a todo vector
        let todo = contents.lines().map(str::to_string).collect();

        // returns todo
        Ok(Self {
            todo,
            todo_path,
            todo_bak,
            no_backup,
        })
    }

    // prints every todo saved
    pub fn list(&self) {
        let stdout = io::stdout();
        // buffered writer for stdout stream
        let mut writer = BufWriter::new(stdout);
        let mut data = String::new();
        // Loops for each task in TODO file
        for (number, task) in self.todo.iter().enumerate() {
            if task.len() > 5 {
                // converts to BOLD string
                let number = (number + 1).to_string().bold();

                // saves the symbol of current task
                let symbol = &task[..4];
                // without symbol of current task
                let task = &task[4..];

                // Checks if current task is complete or not
                if symbol == "[*]" {
                    // Done
                    // if complete print it
                    data = format!("{} {} \n", number, task);
                } else if symbol == "[ ]" {
                    // Not done
                    // if task is not complete print as is
                    data = format!("{} {} \n", number, task);
                }
                writer
                    .write_all(data.as_bytes())
                    .expect("Couldn't write to stdout");
            }
        }
    }

    // Removes a task
    pub fn remove(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo rm takes at least 1 argument");
            process::exit(1);
        }
        // Opens TODO with permission
        let todo_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");

        let mut buffer = BufWriter::new(todo_file);

        for (pos, line) in self.todo.iter().enumerate() {
            if args.contains(&"done".to_string()) && &line[..4] == "[*]" {
                continue;
            }
            if args.contains(&(pos + 1).to_string()) {
                continue;
            }

            let line = format!("{}\n", line);
            buffer
                .write_all(line.as_bytes())
                .expect("Couldn't write to stdout");
        }
    }

    fn remove_file(&self) {
        match fs::remove_file(&self.todo_path) {
            Ok(_) => (),
            Err(e) => eprintln!("Couldn't remove file: {}", e),
        }
    }

    // Clear todo by removing todo file
    pub fn reset(&self) {
        if !self.no_backup {
            match fs::copy(&self.todo_path, &self.todo_bak) {
                Ok(_) => self.remove_file(),
                Err(_) => {
                    eprintln!("Couldn't create backup todo file")
                }
            }
        } else {
            self.remove_file();
        }
    }

    pub fn restore(&self) {
        fs::copy(&self.todo_bak, &self.todo_path).expect("Couldn't restore backup file");
    }

    // sorts completed tasks
    pub fn sort(&self) {
        let mut todo = String::new();
        let mut done = String::new();

        for line in self.todo.iter() {
            if line.len() > 5 {
                if &line[..4] == "[ ]" {
                    let line = format!("{}\n", line);
                    todo.push_str(&line);
                } else if &line[..4] == "[*]" {
                    let line = format!("{}\n", line);
                    done.push_str(&line);
                }
            }
        }
        let newtodo = format!("{}{}", todo, done);

        // opens TODO file with permission
        let mut todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");

        // writes content to new TODO file
        todofile
            .write_all(newtodo.as_bytes())
            .expect("Error while trying to save file");
    }

    pub fn done(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo done takes at least 1 argument");
            process::exit(1);
        }

        // Opens TODO with permissions
        let todo_file = OpenOptions::new()
            .write(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");
        let mut buffer = BufWriter::new(todo_file);

        for (pos, line) in self.todo.iter().enumerate() {
            if line.len() > 5 {
                if args.contains(&(pos + 1).to_string()) {
                    if &line[..4] == "[ ]" {
                        let line = format!("[*] {}\n", &line[4..]);
                        buffer
                            .write_all(line.as_bytes())
                            .expect("Couldn't write to stdout");
                    } else if &line[..4] == "[*]" {
                        let line = format!("[ ] {}\n", &line[4..]);
                        buffer
                            .write_all(line.as_bytes())
                            .expect("Couldn't write to stdout");
                    } else if &line[..4] == "[ ] " || &line[..4] == "[*]" {
                        let line = format!("{}\n", line);
                        buffer
                            .write_all(line.as_bytes())
                            .expect("Couldn't write to stdout");
                    }
                }
            }
        }
    }
}

const TODO_HELP: &str = "Usage: todo [command] [Arguments]
Todo is a super fast and simple tasks organizer written in rust
Example: todo list
Available commands:
    - add [TASK/s]
        adds new task/s
        Example: todo add \"buy carrots\"
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - rm [INDEX]
        removes a task
        Example: todo rm 4
    - reset
        deletes all tasks
    - restore 
        restore recent backup after reset
    - sort
        sorts completed and uncompleted tasks
        Example: todo sort
    - raw [todo/done]
        prints nothing but done/incompleted tasks in plain text, useful for scripting
        Example: todo raw done
";

pub fn help() {
    println!("{}", TODO_HELP)
}