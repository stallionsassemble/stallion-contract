use soroban_sdk::{Address, Env, Symbol, Vec, vec};

pub struct Events;

impl Events {
    fn bounty_created_event(env: &Env) -> Symbol {
        Symbol::new(env, "bounty_created")
    }

    fn bounty_updated_event(env: &Env) -> Symbol {
        Symbol::new(env, "bounty_updated")
    }

    fn bounty_deleted_event(env: &Env) -> Symbol {
        Symbol::new(env, "bounty_deleted")
    }

    fn submission_added_event(env: &Env) -> Symbol {
        Symbol::new(env, "submission_added")
    }
    
    fn submission_updated_event(env: &Env) -> Symbol {
        Symbol::new(env, "submission_updated")
    }

    fn winners_selected_event(env: &Env) -> Symbol {
        Symbol::new(env, "winners_selected")
    }

    fn auto_distributed_event(env: &Env) -> Symbol {
        Symbol::new(env, "auto_distributed")
    }

    fn admin_updated_event(env: &Env) -> Symbol {
        Symbol::new(env, "admin_updated")
    }

    fn fee_account_updated_event(env: &Env) -> Symbol {
        Symbol::new(env, "fee_account_updated")
    }

    pub fn emit_bounty_created(env: &Env, bounty_id: u32) {
        env.events()
            .publish((Self::bounty_created_event(env),), bounty_id);
    }

    pub fn emit_bounty_updated(env: &Env, bounty_id: u32, updated_fields: Vec<Symbol>) {
        env.events().publish(
            (Self::bounty_updated_event(env),),
            vec![env, (bounty_id, updated_fields)],
        );
    }

    pub fn emit_bounty_deleted(env: &Env, bounty_id: u32) {
        env.events()
            .publish((Self::bounty_deleted_event(env),), bounty_id);
    }

    pub fn emit_submission_added(env: &Env, bounty_id: u32, applicant: Address) {
        env.events()
            .publish((Self::submission_added_event(env),), (bounty_id, applicant));
    }
    
    pub fn emit_submission_updated(env: &Env, bounty_id: u32, applicant: Address) {
        env.events()
            .publish((Self::submission_updated_event(env),), (bounty_id, applicant));
    }

    pub fn emit_winners_selected(env: &Env, bounty_id: u32, winners: Vec<Address>) {
        env.events()
            .publish((Self::winners_selected_event(env),), (bounty_id, winners));
    }

    pub fn emit_auto_distributed(env: &Env, bounty_id: u32) {
        env.events()
            .publish((Self::auto_distributed_event(env),), bounty_id);
    }

    pub fn emit_admin_updated(env: &Env, new_admin: Address) {
        env.events()
            .publish((Self::admin_updated_event(env),), new_admin);
    }

    pub fn emit_fee_account_updated(env: &Env, new_fee_account: Address) {
        env.events()
            .publish((Self::fee_account_updated_event(env),), new_fee_account);
    }
}
