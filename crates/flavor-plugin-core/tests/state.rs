use flavor_plugin_core::{PluginCoreConfig, PluginCoreState, Token, TokenCursor};

#[test]
fn config_is_state_injected() {
    let mut config = PluginCoreConfig::default();
    config.snapshot.include_trivia = false;
    let state = PluginCoreState::new(config);

    assert!(!state.config().snapshot.include_trivia);
}

#[test]
fn token_cursor_tracks_position() {
    let token = Token::new("identifier", Default::default());
    let mut cursor = TokenCursor::new(vec![token]);

    assert_eq!(cursor.position(), 0);
    assert!(cursor.peek().is_some());
    assert!(cursor.bump().is_some());
    assert!(cursor.is_at_end());
}
