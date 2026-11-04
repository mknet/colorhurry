import App

extension GameController {
    /// The short mode line in the centre of the watch ring, always on two lines.
    ///
    /// The corner buttons show Skip and Apply, so this line shows only the
    /// choice, the color or the countdown.
    var watchHint: String {
        switch view.screen {
        case .modeSelect:
            guard let choice = view.modeChoice else { return "Mode\nselect" }
            let role = choice.role == .picker ? "Picker" : "Caller"
            return "\(role)\n\(choice.channel)"
        case .picker:
            let color = view.pickerColor?.displayName ?? "?"
            // The watch cannot advertise, so an applied color is not sent.
            return view.pickerApplied ? "Can't send\n\(color)" : "Ch \(view.channel)\n\(color)"
        case .caller:
            return view.countdownActive
                ? "Ch \(view.channel)\n\(view.countdownStep)/10"
                : "Ch \(view.channel)\nwaiting"
        }
    }
}
