from __future__ import annotations

from typing import Any, Callable


CHUNK_LIST_FIELDS = [
    "roads",
    "rails",
    "buildings",
    "water",
    "props",
    "landuse",
    "barriers",
    "rooms",
]

INDEX_ONLY_FIELDS = {
    "partitionVersion",
    "subplans",
    # Build-time filesystem paths -- never shipped to runtime Lua shards.
    "terrainTexturePath",
    "terrainTextureRgbaPath",
}


def base_chunk_fragment(chunk: dict[str, Any]) -> dict[str, Any]:
    fragment: dict[str, Any] = {"id": chunk["id"]}
    for key, value in chunk.items():
        if key == "id":
            continue
        if key in INDEX_ONLY_FIELDS:
            continue
        if key in CHUNK_LIST_FIELDS and isinstance(value, list):
            continue
        if key == "terrain" and isinstance(value, dict):
            terrain_fragment = {
                nested_key: nested_value
                for nested_key, nested_value in value.items()
                if nested_key not in {"heights", "materials"}
            }
            if terrain_fragment:
                fragment[key] = terrain_fragment
            continue
        fragment[key] = value
    return fragment


def chunk_fragment_len(fragment: dict[str, Any], lua_len_fn: Callable[[Any], int]) -> int:
    return lua_len_fn({"chunks": [fragment]})


def fragment_list_payloads(
    *,
    chunk_id: str,
    values: list[Any],
    max_bytes: int,
    field_label: str,
    fragment_builder: Callable[[list[Any]], dict[str, Any]],
    lua_len_fn: Callable[[Any], int],
    lua_value_len_fn: Callable[[Any], int] | None,
    chunk_label: str,
) -> list[dict[str, Any]]:
    fragments: list[dict[str, Any]] = []

    if lua_value_len_fn is not None:
        empty_fragment_len = chunk_fragment_len(fragment_builder([]), lua_len_fn)
        item_lengths = [lua_value_len_fn(value) for value in values]

        start = 0
        current_len = empty_fragment_len
        current_count = 0

        for index, item_len in enumerate(item_lengths):
            next_len = current_len + item_len + (1 if current_count else 0)
            if current_count == 0:
                if next_len > max_bytes:
                    raise SystemExit(
                        f"{chunk_label} {chunk_id} {field_label} contains an entry larger than max bytes {max_bytes}"
                    )
                current_len = next_len
                current_count = 1
                continue

            if next_len > max_bytes:
                fragments.append(fragment_builder(values[start:index]))
                start = index
                current_len = empty_fragment_len + item_len
                current_count = 1
                continue

            current_len = next_len
            current_count += 1

        if current_count:
            fragments.append(fragment_builder(values[start:]))

        return fragments

    start = 0
    while start < len(values):
        low = start + 1
        high = len(values)
        best_end = start

        while low <= high:
            mid = (low + high) // 2
            if chunk_fragment_len(fragment_builder(values[start:mid]), lua_len_fn) <= max_bytes:
                best_end = mid
                low = mid + 1
            else:
                high = mid - 1

        if best_end == start:
            raise SystemExit(
                f"{chunk_label} {chunk_id} {field_label} contains an entry larger than max bytes {max_bytes}"
            )

        fragments.append(fragment_builder(values[start:best_end]))
        start = best_end

    return fragments


def fragment_chunk_for_lua_shards(
    chunk: dict[str, Any],
    max_bytes: int | None,
    *,
    lua_len_fn: Callable[[Any], int],
    lua_value_len_fn: Callable[[Any], int] | None = None,
    chunk_label: str = "chunk",
) -> list[dict[str, Any]]:
    if max_bytes is None:
        return [chunk]

    fragments: list[dict[str, Any]] = []

    base_fragment = base_chunk_fragment(chunk)
    if chunk_fragment_len(base_fragment, lua_len_fn) > max_bytes:
        raise SystemExit(f"{chunk_label} {chunk.get('id')} base metadata exceeds max bytes {max_bytes}")
    fragments.append(base_fragment)

    terrain = chunk.get("terrain")
    if isinstance(terrain, dict):
        for terrain_key in ("heights", "materials"):
            terrain_value = terrain.get(terrain_key)
            if terrain_value is None:
                continue
            if isinstance(terrain_value, list):
                fragments.extend(
                    fragment_list_payloads(
                        chunk_id=chunk["id"],
                        values=terrain_value,
                        max_bytes=max_bytes,
                        field_label=f"terrain field {terrain_key}",
                        fragment_builder=lambda items, terrain_key=terrain_key: {
                            "id": chunk["id"],
                            "terrain": {
                                terrain_key: list(items),
                            },
                        },
                        lua_len_fn=lua_len_fn,
                        lua_value_len_fn=lua_value_len_fn,
                        chunk_label=chunk_label,
                    )
                )
                continue

            fragment = {
                "id": chunk["id"],
                "terrain": {
                    terrain_key: terrain_value,
                },
            }
            if chunk_fragment_len(fragment, lua_len_fn) > max_bytes:
                raise SystemExit(
                    f"{chunk_label} {chunk.get('id')} terrain field {terrain_key} exceeds max bytes {max_bytes}"
                )
            fragments.append(fragment)

    for field in CHUNK_LIST_FIELDS:
        values = chunk.get(field)
        if not isinstance(values, list) or not values:
            continue

        fragments.extend(
            fragment_list_payloads(
                chunk_id=chunk["id"],
                values=values,
                max_bytes=max_bytes,
                field_label=f"field {field}",
                fragment_builder=lambda items, field=field: {"id": chunk["id"], field: list(items)},
                lua_len_fn=lua_len_fn,
                lua_value_len_fn=lua_value_len_fn,
                chunk_label=chunk_label,
            )
        )

    return fragments
