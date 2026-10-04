use crate::{update, view, Event, Model, Screen, ToneKind};
use crate::effects::Effect;

#[test]
fn mode_select_right_enters_picker() {
    let mut model = Model::new();
    let effects = update(Event::ButtonRight, &mut model);
    assert_eq!(model.screen, Screen::Picker);
    assert!(effects.iter().any(|e| matches!(e, Effect::SetBlePicker { .. })));
    assert!(effects.iter().any(|e| matches!(e, Effect::PlayTone(ToneKind::Apply))));
}

#[test]
fn mode_select_left_then_right_can_enter_receiver() {
    let mut model = Model::new();
    // step 0 = picker; advance to step 5 = receiver side
    for _ in 0..5 {
        let _ = update(Event::ButtonLeft, &mut model);
    }
    let effects = update(Event::ButtonRight, &mut model);
    assert_eq!(model.screen, Screen::Receiver);
    assert!(model.awaiting_entropy);
    assert!(effects.iter().any(|e| matches!(e, Effect::ClearBle)));
}

#[test]
fn receiver_entropy_starts_countdown() {
    let mut model = Model::new();
    for _ in 0..5 {
        let _ = update(Event::ButtonLeft, &mut model);
    }
    let _ = update(Event::ButtonRight, &mut model);
    let effects = update(Event::Entropy(3), &mut model);
    assert!(model.countdown_active);
    assert!(!model.awaiting_entropy);
    assert!(effects
        .iter()
        .any(|e| matches!(e, Effect::SetBleReceiver { .. })));
    let vm = view(&model);
    assert!(vm.leds.iter().any(|c| *c != crate::Rgb::OFF));
}

#[test]
fn receiver_match_aborts_without_explosion() {
    let mut model = Model::new();
    for _ in 0..5 {
        let _ = update(Event::ButtonLeft, &mut model);
    }
    let _ = update(Event::ButtonRight, &mut model);
    let _ = update(Event::Entropy(1), &mut model);
    let target = model.receiver_target;
    let ch = model.channel();
    let effects = update(
        Event::BleColorReceived {
            channel: ch,
            color: target,
        },
        &mut model,
    );
    assert!(!model.countdown_active);
    assert!(model.awaiting_entropy);
    assert!(!effects
        .iter()
        .any(|e| matches!(e, Effect::PlayTone(ToneKind::Explosion))));
}

#[test]
fn picker_apply_broadcasts_color() {
    let mut model = Model::new();
    let _ = update(Event::ButtonRight, &mut model); // enter picker
    let effects = update(Event::ButtonRight, &mut model); // apply
    assert!(effects.iter().any(|e| matches!(
        e,
        Effect::SetPickerBroadcast {
            color: Some(_)
        }
    )));
}
