//! Stepping the current time from the keyboard: Page Down / Page Up and Cmd+Right / Cmd+Left
//! Arrow move one frame, with Shift ten.

use effectcraft_engine::Session;
use effectcraft_engine::project::ItemId;
use effectcraft_ui_egui::EffectcraftApp;
use effectcraft_ui_egui::dock::PanelKind;
use egui::{Event, Key, Modifiers, PointerButton};
use egui_kittest::Harness;
use serde_json::json;

fn key(h: &mut Harness<'_, EffectcraftApp>, key: Key, modifiers: Modifiers) {
    h.event(Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers });
    h.step();
    h.event(Event::Key { key, physical_key: None, pressed: false, repeat: false, modifiers });
    h.step();
}

#[test]
fn cmd_arrows_step_the_current_frame() {
    let mut session = Session::default();
    session.execute("comp.new", json!({"name": "Main", "width": 320, "height": 180, "frameRate": 30.0, "duration": 4.0})).unwrap();
    let cid = session.state.active_comp.unwrap();
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_| EffectcraftApp::new(session));
    h.run_steps(2);
    let frame = |h: &Harness<'_, EffectcraftApp>| {
        let app = h.state();
        app.session.project.comp(cid).unwrap().frame_rate.frame_at(app.session.time())
    };
    assert_eq!(frame(&h), 0);
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
    key(&mut h, Key::ArrowRight, cmd);
    assert_eq!(frame(&h), 1);
    key(&mut h, Key::PageDown, Modifiers::NONE);
    assert_eq!(frame(&h), 2);
    key(&mut h, Key::ArrowRight, cmd_shift);
    assert_eq!(frame(&h), 12);
    key(&mut h, Key::ArrowLeft, cmd);
    assert_eq!(frame(&h), 11);
    key(&mut h, Key::ArrowLeft, cmd_shift);
    assert_eq!(frame(&h), 1);
    // The plain arrows still nudge layers, not the time.
    key(&mut h, Key::ArrowRight, Modifiers::NONE);
    assert_eq!(frame(&h), 1);
}

fn harness(duration: f64) -> (Harness<'static, EffectcraftApp>, ItemId) {
    let mut session = Session::default();
    session.execute("comp.new", json!({"name": "Main", "width": 320, "height": 180, "frameRate": 30.0, "duration": duration})).unwrap();
    let cid = session.state.active_comp.unwrap();
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_| EffectcraftApp::new(session));
    h.run_steps(2);
    (h, cid)
}

fn frame(h: &Harness<'_, EffectcraftApp>, cid: ItemId) -> i64 {
    let app = h.state();
    app.session.project.comp(cid).unwrap().frame_rate.frame_at(app.session.time())
}

fn rect(h: &Harness<'_, EffectcraftApp>, id: &str) -> egui::Rect {
    let e = h.state().auto.find(id).unwrap_or_else(|| panic!("no {id}"));
    egui::Rect::from_min_size(egui::pos2(e.rect[0], e.rect[1]), egui::vec2(e.rect[2], e.rect[3]))
}

fn click(h: &mut Harness<'_, EffectcraftApp>, p: egui::Pos2) {
    h.event(Event::PointerMoved(p));
    h.event(Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.step();
    h.event(Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(2);
}

/// The keys stop at the comp's first and last frames, as After Effects' do.
#[test]
fn cmd_arrows_stop_at_the_first_and_last_frame() {
    let (mut h, cid) = harness(1.0);
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
    key(&mut h, Key::ArrowLeft, cmd);
    assert_eq!(frame(&h, cid), 0);
    key(&mut h, Key::ArrowLeft, cmd_shift);
    assert_eq!(frame(&h, cid), 0);
    key(&mut h, Key::ArrowRight, cmd_shift);
    key(&mut h, Key::ArrowRight, cmd_shift);
    assert_eq!(frame(&h, cid), 20);
    key(&mut h, Key::ArrowRight, cmd_shift);
    assert_eq!(frame(&h, cid), 29, "30 frames: the last is 29");
    key(&mut h, Key::ArrowRight, cmd);
    assert_eq!(frame(&h, cid), 29);
}

/// Cmd+arrows in a text field edit the text: the time stays put.
#[test]
fn cmd_arrows_in_a_text_field_leave_the_time_alone() {
    let (mut h, cid) = harness(4.0);
    h.state_mut().session.execute("time.set", json!({"frame": 5})).unwrap();
    h.run_steps(2);
    let search = rect(&h, "timeline.search").center();
    click(&mut h, search);
    h.event(Event::Text("scale".into()));
    h.run_steps(2);
    for m in [Modifiers::COMMAND, Modifiers::COMMAND | Modifiers::SHIFT] {
        key(&mut h, Key::ArrowLeft, m);
        key(&mut h, Key::ArrowRight, m);
    }
    assert_eq!(frame(&h, cid), 5, "the Timeline search field kept the keys");
}

/// While typing into a text layer in the viewer, Cmd+Right / Cmd+Left jump by word.
#[test]
fn cmd_arrows_move_the_caret_while_editing_text() {
    let (mut h, cid) = harness(4.0);
    h.state_mut().show_panel(PanelKind::Composition);
    h.run_steps(3);
    h.state_mut().session.execute("time.set", json!({"frame": 5})).unwrap();
    let ctx = h.ctx.clone();
    effectcraft_ui_egui::menus::invoke(h.state_mut(), &ctx, "layer.newText", json!({})).unwrap();
    h.run_steps(2);
    h.event(Event::Text("alpha beta".into()));
    h.run_steps(2);
    key(&mut h, Key::ArrowLeft, Modifiers::COMMAND);
    let caret = h.state().session.state.text_edit.as_ref().map(|e| e.caret);
    assert_eq!(caret, Some(6), "word left");
    key(&mut h, Key::ArrowRight, Modifiers::COMMAND | Modifiers::SHIFT);
    assert_eq!(frame(&h, cid), 5, "the caret moved, not the time");
}

/// Alt+Right / Alt+Left still nudge the selected keyframes a frame and leave the time alone.
#[test]
fn alt_arrows_still_nudge_keyframes() {
    let (mut h, cid) = harness(4.0);
    let s = &mut h.state_mut().session;
    let layer = s.execute("layer.newSolid", json!({"color": "#ffffff"})).unwrap()["layer"].as_u64().unwrap();
    s.execute("prop.addKey", json!({"layer": layer, "path": "transform/opacity", "time": 1.0, "value": 0})).unwrap();
    s.execute("keys.select", json!({"keys": [{"layer": layer, "path": "transform/opacity", "time": 1.0}]})).unwrap();
    s.execute("time.set", json!({"frame": 5})).unwrap();
    h.run_steps(2);
    let key_frame = |h: &Harness<'_, EffectcraftApp>| {
        let comp = h.state().session.project.comp(cid).unwrap();
        let l = comp.layers.iter().find(|l| l.id.0 == layer).unwrap();
        comp.frame_rate.frame_at(l.comp_time(l.props.prop("transform/opacity").unwrap().keys[0].time))
    };
    assert_eq!(key_frame(&h), 30);
    key(&mut h, Key::ArrowRight, Modifiers::ALT);
    assert_eq!(key_frame(&h), 31);
    key(&mut h, Key::ArrowLeft, Modifiers::ALT);
    key(&mut h, Key::ArrowLeft, Modifiers::ALT);
    assert_eq!(key_frame(&h), 29);
    assert_eq!(frame(&h, cid), 5, "Alt+arrows move keys, not the time");
}
