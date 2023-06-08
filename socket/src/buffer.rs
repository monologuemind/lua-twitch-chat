use neovim_lib::{Neovim, NeovimApi, Value};

pub fn create_buffer(nvim: &mut Neovim) -> Option<i64> {
    let buf = nvim.session.call(
        "nvim_create_buf",
        vec![Value::from(true), Value::from(false)],
    );

    if let Err(e) = buf.clone() {
        nvim.command(&format!("echo \"Error creating buf: {}\"", e.to_string()))
            .unwrap();
        return None;
    }

    let value = buf.unwrap();

    let res = nvim.session.call(
        "nvim_buf_set_name",
        vec![
            Value::from(value.clone()),
            Value::from("some_name".to_string()),
        ],
    );

    if let Err(e) = res {
        nvim.command(&format!("echo \"Error setting name: {}\"", e.to_string()))
            .unwrap();
        return None;
    }

    nvim.command("vsplit").unwrap();

    let res = nvim
        .session
        .call("nvim_win_set_buf", vec![Value::from(0), value.clone()]);

    if let Err(e) = res {
        nvim.command(&format!("echo \"Error setting buf: {}\"", e.to_string()))
            .unwrap();
        return None;
    }

    return value.as_i64();
}

pub fn append_rows(nvim: &mut Neovim, buf: i64, more_lines: Vec<Value>) {
    let res = nvim
        .session
        .call("nvim_buf_line_count", vec![Value::from(buf)]);

    if let Err(e) = res {
        nvim.command(&format!(
            "echo \"Error getting buf_line_count: {}\"",
            e.to_string()
        ))
        .unwrap();
        return;
    }

    let buf_line_count = res.unwrap();
    // TODO(Buser): If buf_line_count => 10,000
    // Flush the table (into file if set to store chat logs)
    // The append message to top at 0, 0 (requires setting buf_line_count to 0)

    let res = nvim.session.call(
        "nvim_buf_set_lines",
        vec![
            Value::from(buf),
            buf_line_count.clone(),
            buf_line_count.clone(),
            Value::from(true),
            Value::from(more_lines),
        ],
    );

    if let Err(e) = res {
        nvim.command(&format!("echo \"Error append_rows: {}\"", e.to_string()))
            .unwrap();
        return;
    }
}

// pub fn show_buffer(nvim: &mut Neovim, buf: i64) {
//     let res = nvim.session.call(
//         "nvim_buf_set_var",
//         vec![Value::from(buf), Value::from("&hidden"), Value::from(1)],
//     );
//
//     if let Err(e) = res {
//         nvim.command(&format!("echo \"Error show_buffer: {}\"", e.to_string()))
//             .unwrap();
//         return;
//     }
// }
//
// pub fn hide_buffer(nvim: &mut Neovim, buf: i64) {
//     let res = nvim.session.call(
//         "nvim_buf_set_var",
//         vec![Value::from(buf), Value::from("&hidden"), Value::from(0)],
//     );
//
//     if let Err(e) = res {
//         nvim.command(&format!("echo \"Error hide_buffer: {}\"", e.to_string()))
//             .unwrap();
//         return;
//     }
// }
