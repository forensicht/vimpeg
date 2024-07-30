use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    adw::prelude::ApplicationExt,
    main_adw_application,
};

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(QuitAction, AppActionGroup, "quit");

pub(crate) fn init() {
    let app = main_adw_application();
    app.set_accelerators_for_action::<QuitAction>(&["<Control>q"]);

    let quit_action = {
        let app = app.clone();
        RelmAction::<QuitAction>::new_stateless(move |_| {
            app.quit();
        })
    };

    let mut actions = RelmActionGroup::<AppActionGroup>::new();
    actions.add_action(quit_action);
    actions.register_for_main_application();
}
