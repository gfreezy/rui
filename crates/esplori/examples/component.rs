struct Context;
struct Todo {
    text: String,
    done: bool,
}

fn todo(todo: String, done: bool) {}

#[memoize]
fn todos(context: Context, todos: Vec<Todo>) {
    fn inner(context: Context, todos: Vec<Todo>) {
        for t in todos {
            todo(t.text, t.done);
        }
    }

    context.on_change(inner)
}
