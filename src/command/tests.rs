use super::*;

#[test]
fn test_named_logged_block_command() {
    // given:
    let name = "Herobrine";
    let command = "say hi";

    // when:
    let actual = named_logged_block_command(name, command);

    // then:
    assert!(actual.chars().all(|c| c != '\n'));
}

#[test]
fn test_named_logged_cart_command() {
    // given:
    let name = "Herobrine";
    let command = "say hi";

    // when:
    let actual = named_logged_cart_command(name, command);

    // then:
    assert!(actual.chars().all(|c| c != '\n'));
}
