# Schema Migration Guide

Relish-based systems have **writers** (serialize messages) and **readers** (deserialize messages). In distributed systems, these update independently. A migration is safe when old and new versions of both readers and writers interoperate correctly.

## Adding a Field

Add new fields as `Option<T>`:

```rust
struct Request {
    #[relish(field_id = 0)]
    user_id: u64,
    #[relish(field_id = 1)]
    trace_id: Option<String>,  // New field
}
```

Old readers skip unknown fields. New readers receive `None` from old writers.

Once all writers supply a non-`None` `trace_id`, you may change the field to required.

## Removing a Field

Either make the field `Option<T>` on all readers before writers stop sending it, or stop using the field's value in reader code before writers stop sending it.

Old writers continue sending the field; new readers must tolerate its presence. New writers stop sending; old readers must tolerate its absence.

## Changing a Field's Type

Type changes cannot be done in-place. Add a new field with the new type:

```rust
struct Event {
    #[relish(field_id = 0)]
    timestamp_secs: Option<u32>,   // Old
    #[relish(field_id = 1)]
    timestamp_millis: Option<u64>, // New
}
```

1. Deploy readers that handle both fields (prefer new, fall back to old)
2. Deploy writers that send both fields
3. Remove the old field from writers once all readers are updated
4. Remove the old field from readers once all writers are updated

## Renaming a Field

No action required. Fields are identified by `field_id`, not name.

## Reordering Fields

No action required. Field order in source code does not affect the wire format.

## Adding Enum Variants

Unlike struct fields, unknown enum variants cause parse errors. There, you must deploy all readers with the new variant before any writer sends it.
