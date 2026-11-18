# fs-components-catalog: what a building block accepts

**This is a building block. It is why a frontend described in fs-forms can bind a board, a form field or a menu to any program's data and be told, before anything is drawn, which part does not fit and why.**

The block contract of fs-components (0.4) gives every block the key `accepts`: its slots. A slot says what it takes: a value (basic types, role input, display, compact or cell, single or multiple), the records of a type, a grouping, an order, a command or a lock; whether it is required, which records slot it belongs to (`within`) and how often it may be bound (`repeat`). Frames without data have `accepts = []`.

`BlockCatalog::read` reads the contract files; `check` tests the bindings of one block against the block, the provider's type catalog (`TypeCatalog.describe`) and its record shapes (`Records.shape`, crate `fs-records`). Every problem is returned as a code with parameters (`accepts-error-*`), never a sentence; an empty result means the block can be drawn from the description alone. A binding names a capability with a version range (`tasks >=0.2 <0.3`), never a program.

Since 0.4 a block that draws a field also says `renders` (basic type, the targets of a reference, the roles), and `BlockCatalog::choose(field, role)` returns the one block for it: the host keeps no table of types and blocks, and a style never changes the choice. No block for a type and role is the visible code `renders-none`; a type this reader does not know is drawn by `field-text`, locked.

Written with the help of Claude (Anthropic). License: MIT OR Apache-2.0.
