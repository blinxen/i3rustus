use crate::widgets::{Widget, WidgetError};
use actix::prelude::*;
use serde_json::Value;

// Actix message that is used to start a "update" job
#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateWidgetValue;

// Actix message that is used to get the value of a widget
#[derive(Message)]
#[rtype(result = "Result<Value, WidgetError>")]
pub struct WidgetValue;

// A widget executor runs a widget without blocking the main thread
// and calls the update method asynchronously
pub struct WidgetExecutor {
    // Widget that should be executed in background
    widget: Box<dyn Widget>,
}

impl WidgetExecutor {
    // The widget has a static lifetime because we return a reference to it in `fn widget()`
    // The lifetime has to be static because this widget will live as long as the program (maybe
    // process is the better word?) is running
    pub fn new(widget: Box<dyn Widget>) -> Self {
        Self { widget }
    }
}

impl Actor for WidgetExecutor {
    type Context = Context<Self>;
}

impl Handler<UpdateWidgetValue> for WidgetExecutor {
    type Result = ();

    fn handle(&mut self, _msg: UpdateWidgetValue, _ctx: &mut Context<Self>) {
        self.widget.update();
    }
}

impl Handler<WidgetValue> for WidgetExecutor {
    type Result = Result<Value, WidgetError>;

    fn handle(&mut self, _msg: WidgetValue, _ctx: &mut Context<Self>) -> Self::Result {
        self.widget.display_text()
    }
}
