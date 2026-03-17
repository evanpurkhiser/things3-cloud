from tests.helpers import get_fixture, run_cli


def _tag_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    shortcut: str | None = None,
) -> dict:
    props = {"tt": title, "ix": ix}
    if shortcut is not None:
        props["sh"] = shortcut
    return {uuid: {"t": 0, "e": "Tag4", "p": props}}


def test_tags_empty(store_from_journal) -> None:
    assert run_cli("tags", store_from_journal([])) == get_fixture("tags_empty")


def test_tags_basic_list(store_from_journal) -> None:
    journal = [
        _tag_create("GKYVAxEFFoZX9qRavLpSxC", "Home", ix=10),
        _tag_create("5QpptG3mkc9Euc372cZH2X", "Work", ix=20),
    ]

    assert run_cli("tags", store_from_journal(journal)) == get_fixture(
        "tags_basic_list"
    )


def test_tags_renders_shortcuts(store_from_journal) -> None:
    journal = [
        _tag_create("HJXTqkytEmD1tFNQboJbaK", "Focus", ix=10, shortcut="f"),
        _tag_create("Ai9KrPNZbVwf5VFKDMNLc7", "Home", ix=20),
        _tag_create("DkUdPWL22mk5bkFr5Y7q6t", "Errands", ix=30, shortcut="e"),
    ]

    assert run_cli("tags", store_from_journal(journal)) == get_fixture(
        "tags_shortcut_rendering"
    )


def test_tags_filters_blank_and_whitespace_titles(store_from_journal) -> None:
    journal = [
        _tag_create("5DpSbPqqW43rGrmHeTtpYC", "", ix=5),
        _tag_create("V6ovbTrWN2p5yCNo3GaNPS", "   ", ix=10),
        _tag_create("SodAejXUasPJGBoLKdJ7hy", "Errands", ix=15),
        _tag_create("Ka2MbPmKkLgmR3v3jE6LU9", "\t", ix=20),
        _tag_create("QfcNkgb1LwJqmUyXQhcwGN", "Personal", ix=25),
    ]

    assert run_cli("tags", store_from_journal(journal)) == get_fixture(
        "tags_filter_blank_titles"
    )
