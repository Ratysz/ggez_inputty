/// Generates implementations of input-related methods of `ggez::event::EventHandler`.
#[macro_export]
macro_rules! impl_input_handling {
    ($handler:ident) => {
        fn mouse_button_down_event(
            &mut self,
            _ctx: &mut ggez::Context,
            button: ggez::event::MouseButton,
            x: i32,
            y: i32,
        ) {
            self.$handler.mouse_button_down_event(button, x, y);
        }

        fn mouse_button_up_event(
            &mut self,
            _ctx: &mut ggez::Context,
            button: ggez::event::MouseButton,
            x: i32,
            y: i32,
        ) {
            self.$handler.mouse_button_up_event(button, x, y);
        }

        fn mouse_motion_event(
            &mut self,
            _ctx: &mut ggez::Context,
            state: ggez::event::MouseState,
            x: i32,
            y: i32,
            xrel: i32,
            yrel: i32,
        ) {
            self.$handler.mouse_motion_event(state, x, y, xrel, yrel);
        }

        fn mouse_wheel_event(
            &mut self,
            _ctx: &mut ggez::Context,
            x: i32,
            y: i32,
        ) {
            self.$handler.mouse_wheel_event(x, y);
        }

        fn key_down_event(
            &mut self,
            _ctx: &mut ggez::Context,
            keycode: ggez::event::Keycode,
            keymod: ggez::event::Mod,
            repeat: bool,
        ) {
            self.$handler.key_down_event(keycode, keymod, repeat);
        }

        fn key_up_event(
            &mut self,
            _ctx: &mut ggez::Context,
            keycode: ggez::event::Keycode,
            keymod: ggez::event::Mod,
            repeat: bool,
        ) {
            self.$handler.key_up_event(keycode, keymod, repeat);
        }

        fn controller_button_down_event(
            &mut self,
            _ctx: &mut ggez::Context,
            button: ggez::event::Button,
            instance_id: i32,
        ) {
            self.$handler.controller_button_down_event(button, instance_id);
        }

        fn controller_button_up_event(
            &mut self,
            _ctx: &mut ggez::Context,
            button: ggez::event::Button,
            instance_id: i32,
        ) {
            self.$handler.controller_button_up_event(button, instance_id);
        }

        fn controller_axis_event(
            &mut self,
            _ctx: &mut ggez::Context,
            axis: ggez::event::Axis,
            value: i16,
            instance_id: i32,
        ) {
            self.$handler.controller_axis_event(axis, value, instance_id);
        }
    };
}
