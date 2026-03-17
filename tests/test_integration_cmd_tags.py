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
        _tag_create("tag-home-0001", "Home", ix=10),
        _tag_create("tag-work-0002", "Work", ix=20),
    ]

    assert run_cli("tags", store_from_journal(journal)) == get_fixture(
        "tags_basic_list"
    )


def test_tags_renders_shortcuts(store_from_journal) -> None:
    journal = [
        _tag_create("tag-focus-0001", "Focus", ix=10, shortcut="f"),
        _tag_create("tag-home-0002", "Home", ix=20),
        _tag_create("tag-errands-03", "Errands", ix=30, shortcut="e"),
    ]

    assert run_cli("tags", store_from_journal(journal)) == get_fixture(
        "tags_shortcut_rendering"
    )


def test_tags_filters_blank_and_whitespace_titles(store_from_journal) -> None:
    journal = [
        _tag_create("tag-empty-0001", "", ix=5),
        _tag_create("tag-space-0002", "   ", ix=10),
        _tag_create("tag-valid-0003", "Errands", ix=15),
        _tag_create("tag-tab-00004", "\t", ix=20),
        _tag_create("tag-valid-0005", "Personal", ix=25),
    ]

    assert run_cli("tags", store_from_journal(journal)) == get_fixture(
        "tags_filter_blank_titles"
    )
