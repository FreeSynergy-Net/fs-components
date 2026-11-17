# DRAFT (trait-delta fs-settings, Opus 2026-11-17) for fs-interfaces `schema/settings.capnp` (Folge-Lauf 1).
# STATUS: DRAFT (ADR 0045). Wire form of the crate fs-setting for fs-connect (the configurator, a server instance, MCP
# with its own actor). Typed, no JSON (ADR 0086). Not compiled; the probe builds Rust only.
@0xd5c1a7e3b2f40961;  # placeholder id; the Bau-Lauf generates a real one with `capnp id`

using Common = import "common.capnp";

struct Raw {                         # crate `Raw`: what a store keeps
  union {
    flag  @0 :Bool;
    whole @1 :Int64;
    text  @2 :Text;
  }
}

enum Layer { default @0; instance @1; system @2; person @3; program @4; }
enum Place { profile @0; device @1; }
enum Target { thisProgram @0; allPrograms @1; }

struct Kind {
  union {
    flag     @0 :Void;
    whole    @1 :Range;
    choice   @2 :List(Text);
    language @3 :Void;
    text     @4 :UInt32;             # max chars
  }
  struct Range { min @0 :Int64; max @1 :Int64; }
}

struct Declaration {                 # crate `Declaration`; label/about = MF2 keys derived from `key`
  key     @0 :Text;
  kind    @1 :Kind;
  default @2 :Raw;
  group   @3 :Text;
  place   @4 :Place;
}

struct Lock { reason @0 :Text; lockedBy @1 :Text; lockedAt @2 :Int64; }   # UTC seconds

struct Effective { key @0 :Text; value @1 :Raw; from @2 :Layer; lock @3 :Lock; locked @4 :Bool; canLock @5 :Bool; }

interface Settings {                 # one per program and person; rights come from fs-access
  catalog @0 () -> (all :List(Declaration));
  get     @1 (keys :List(Text)) -> (values :List(Effective));
  save    @2 (key :Text, value :Raw, reset :Bool, target :Target) -> ();    # errors: Common.Problem with the code
  watch   @3 (listener :Listener) -> (subscription :Common.Subscription);
  interface Listener { changed @0 (change :Effective) -> stream; }
}

interface Instance {                 # the admin's side; every call checks `can_lock` / the admin right
  setDefault @0 (key :Text, value :Raw, reset :Bool) -> ();
  lock       @1 (key :Text, value :Raw, reason :Text) -> ();   # lockedBy/lockedAt set by the server, never by the caller
  unlock     @2 (key :Text) -> ();
}
