use bevy_ecs::prelude::*;
use super::super::systems::*;

#[derive(StageLabel)]
enum Labels {
    ProcessNetwork,
    Find,
    Action,
    Consequence,
    Death,
    NetworkOut,
    BookeepingUpdatedGO,
    BookeepingRemovedGO
}

pub fn generate_schedule() -> Schedule {
    let mut s = Schedule::default();
    
    // distribute network inputs
    let mut network_stage = SystemStage::parallel();
    network_stage.add_system(navigation::sys_process_navigation_inputs_local);
    network_stage.add_system(navigation::sys_process_navigation_inputs_warp);
    network_stage.add_system(sense::sys_get_visible); // not sure if it makes sense to do this here
    network_stage.add_system(market::sys_process_market); // want this to process before inventory motion later
    network_stage.add_system(hanger_mgmt::hanger_mgmt); // this can process at the same time as the market, but not at the same time as inventory management
    network_stage.add_system(inventory_mgmt::sys_inventory_service_inventory_requests); // this actually does a bit of heavy lifting to grab stations from inventory IDs

    // entities examining other entities find them and collect the info they want (before it gets mutated)
    let mut find_stage = SystemStage::parallel();
    find_stage.add_system(navigation::sys_navigation_update_transform_positions);

    // entities asses their current state (collected from the above stages), and start sending out update messages
    let mut action_stage = SystemStage::parallel();
    action_stage.add_system(navigation::sys_tick_navigation);
    action_stage.add_system(jump::sys_process_jump_inputs);
    action_stage.add_system(docking_undocking::sys_process_dock);
    action_stage.add_system(inventory_mgmt::sys_manage_inventory_transfers);

    // entities receive updates messages and apply them to themselves
    let mut consequence_stage = SystemStage::parallel();
    consequence_stage.add_system(navigation::sys_tick_transforms);

    // things that might die get checked for death here, and scheduled for kill if needed
    let mut death_stage = SystemStage::parallel();

    death_stage.add_system(logon_mgmt::sys_dispatch_login_info);

    // sends messages to everyone about what happened
    let mut network_out_stage = SystemStage::parallel();
    network_out_stage.add_system(network_msg_generator::sys_dispatch_static_data);
    network_out_stage.add_system(network_msg_generator::sys_dispatch_other_ships);
    network_out_stage.add_system(network_msg_generator::sys_dispatch_own_ship);
    network_out_stage.add_system(network_msg_generator::sys_dispatch_ev_dock_undock_jump);
    network_out_stage.add_system(network_msg_generator::sys_dispatch_inv_bank_updates);
    network_out_stage.add_system(network_msg_generator::sys_dispatch_ship_inventory_requests);
    network_out_stage.add_system(network_msg_generator::sys_dispatch_other_ships_movement);
    network_out_stage.add_system(network_msg_generator::sys_dispatch_own_ship_movement);

    // all the bookkeeping for jumps, docks, and undocks is handled here
    let mut update_stage = SystemStage::parallel();
    update_stage
        .add_system(path_table_bookeeping::update_path_table)
        .add_system(star_system_table_bookeeping::update_star_system_table)
        .add_system(logon_mgmt::logon_bookeeping_handle_send_initial_info);

    // all the bookkeeping for things that died is handled here
    let mut removal_stage = SystemStage::parallel();
    removal_stage
        .add_system(removal_hooks::process_removals_star_system_table);

    s.add_stage(Labels::ProcessNetwork, network_stage);
    s.add_stage_after(Labels::ProcessNetwork, Labels::Find, find_stage);
    s.add_stage_after(Labels::Find, Labels::Action, action_stage);
    s.add_stage_after(Labels::Action, Labels::Consequence, consequence_stage);
    s.add_stage_after(Labels::Consequence, Labels::Death, death_stage);
    s.add_stage_after(Labels::Death, Labels::NetworkOut, network_out_stage);
    s.add_stage_after(Labels::NetworkOut, Labels::BookeepingUpdatedGO, update_stage);
    s.add_stage_after(Labels::BookeepingUpdatedGO, Labels::BookeepingRemovedGO, removal_stage);
        
    s
}