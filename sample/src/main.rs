use neovim_lib::{Neovim, NeovimApi, Session};

enum Messages {
    Test,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "test" => Messages::Test,
            _ => Messages::Unknown(event),
        }
    }
}

struct EventHandler {
    nvim: Neovim,
}

impl EventHandler {
    fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        EventHandler { nvim }
    }

    fn recv(&mut self) {
        let receiver = self.nvim.session.start_event_loop_channel();
        let _ = self.nvim.command("echo \"sup\"");

        for (event, _) in receiver {
            match Messages::from(event) {
                Messages::Test => {
                    self.nvim.command("echo \"testing\"").unwrap();
                }
                Messages::Unknown(event) => {
                    self.nvim
                        .command(&format!(
                            "echoerr \"Unknown command: {}. We support Init, Join, Send, OAuth\"",
                            event
                        ))
                        .unwrap();
                }
            }
        }
    }
}

fn main() {
    let mut evt_hdl = EventHandler::new();

    evt_hdl.recv();
}
