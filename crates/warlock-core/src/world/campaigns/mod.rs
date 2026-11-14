//! The campaigns' places and people, end to end.

cfg_if! {
    if #[cfg(test)] {
        mod ascmearc_encounter_tests;
        mod ascmearc_tests;
        mod ceaptun_quest_tests;
        mod ceaptun_tests;
        mod crystal_guardian_tests;
        mod deep_gnome_tests;
        mod druid_quest_tests;
        mod gnome_letter_tests;
        mod hellegeat_tests;
        mod holtwic_tests;
        mod lictun_tests;
        mod pit_warden_tests;
        mod scriptorium_tests;
        mod strange_spirit_tests;
        mod wynflaed_quest_tests;
    }
}
