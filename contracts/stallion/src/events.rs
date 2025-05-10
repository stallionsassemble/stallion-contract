use soroban_sdk::{Address, Env, Symbol, Vec};

pub struct Events;

impl Events {
    fn bounty_created_event(env: &Env) -> Symbol {
        Symbol::new(env, "bounty_created")
    }

    fn submission_added_event(env: &Env) -> Symbol {
        Symbol::new(env, "submission_added")
    }

    fn winners_selected_event(env: &Env) -> Symbol {
        Symbol::new(env, "winners_selected")
    }

    fn auto_distributed_event(env: &Env) -> Symbol {
        Symbol::new(env, "auto_distributed")
    }

    pub fn bounty_created(env: &Env, bounty_id: u32) {
        env.events()
            .publish((Self::bounty_created_event(env),), bounty_id);
    }

    pub fn submission_added(env: &Env, bounty_id: u32, applicant: Address) {
        env.events()
            .publish((Self::submission_added_event(env),), (bounty_id, applicant));
    }

    pub fn winners_selected(env: &Env, bounty_id: u32, winners: Vec<Address>) {
        env.events()
            .publish((Self::winners_selected_event(env),), (bounty_id, winners));
    }

    pub fn auto_distributed(env: &Env, bounty_id: u32) {
        env.events()
            .publish((Self::auto_distributed_event(env),), bounty_id);
    }
}
