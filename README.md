# Requirements

- Rust 1.80.0 stable (install with `rustup toolchain install stable`)
- [`cargo-tarpaulin`](https://github.com/xd009642/tarpaulin) (Code coverage
  reporter, install with `cargo install cargo-tarpaulin`)
- A containerization tool supporting the compose spec
  ([`podman compose`](https://github.com/containers/podman-compose) or
  [`docker compose`](https://github.com/docker/compose) for example)
- The [`sqlx cli`](https://crates.io/crates/sqlx-cli) (install with
  `cargo install sqlx-cli`); Necesarry to run database migrations

## Dependencies

I will explain here the different choices for the dependencies of this project:

### [`chrono`](https://crates.io/crates/chrono)

Chrono is the golden standard for datetimes and timezones in rust, it's
integrated with the entire ecosystem and is really simple to use.

### [`clap`](https://crates.io/crates/clap)

Clap is the library used to parse CLI arguments and map them to a rust struct.

### [`thiserror`](https://crates.io/crates/thiserror)

Thiserror facilitates the use of error enumerations, while still implementing
standard error trait `std::error::Error`. This means our errors can benefit from
the rust ecosystem with crates like [`anyhow`](https://crates.io/crates/anyhow)
or [`eyre`](https://crates.io/crates/eyre)

### [`uuid`](https://crates.io/crates/uuid)

Same as chrono, golden standard for doing it's thing (so uuid in this case);
implements all versions of the uuid spec from 1 to 8 and is dead simple to use.

## What I could've used instead

And now here are some crates that I could have used for this project, but made
the choice not to. I'm mostly doing this to explain my mental path to getting to
were this project is at.

### [`jiff`](https://crates.io/crates/jiff)

New datetime library, created as an alternative to chrono. It is developped by a
BurntSushi (creator of [ripgrep](https://github.com/BurntSushi/ripgrep),
maintainer of the [`regex`](https://crates.io/crates/regex) crate). Though it's
still pretty new, and isn't supported by most libraries, and isn't even stable
yet so it was pretty clear I wasn't going to use it.

### [`nutype`](https://crates.io/crates/nutype)

Honestly I forgot this one existed until near the end of the project so I didn't
include, but I would've definitely used it if I had to start over. Nutype allows
creating
[_newtype structs_](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
really easily. It helps validating inputs in a struct and returning an error
when necessary. This is done manually right now which is pretty redundant since
we'll have the same kind of checks in a lot of other places. With nutype, we can
transform this code:

```rust
pub const POSITION_PRICE_MIN_IN_CENTS: u32 = 300;
pub const POSITION_PRICE_MAX_IN_CENTS: u32 = 800;

#[derive(Eq, PartialEq, Debug)]
pub struct PositionPrice {
    cents: u32,
}

impl PositionPrice {
    pub fn from_cents(cents: u32) -> Result<Self, PositionPriceError> {
        use PositionPriceError::*;

        if cents < POSITION_PRICE_MIN_IN_CENTS || cents > POSITION_PRICE_MAX_IN_CENTS {
            return Err(OutOfBounds);
        }

        Ok(Self { cents })
    }

    pub fn to_cents(&self) -> u32 {
        self.cents
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PositionPriceError {
    #[error("Position price should be between {} and {}", (POSITION_PRICE_MIN_IN_CENTS as f32) / 100.0, (POSITION_PRICE_MAX_IN_CENTS as f32) / 100.0)]
    OutOfBounds,
}
```

Into something like this:

```rust
#[nutype::nutype(
    validator(less=POSITION_PRICE_MIN_IN_CENTS ,greater=POSITION_PRICE_MAX_IN_CENTS)
    derive(Eq, PartialEq, Debug)
)]
pub struct PositionPrice(u32);
```

# Setting up the project

```bash
git clone git@github.com:Zuruuh/sora.git
cd sora
docker compose up -d
sqlx migrate run
cargo run -- --help
```

# Examples

**todo add cli examples here**

# Testing

This project uses the [rstest](https://crates.io/crates/rstest) crate to add
extensive scenarios to tests. You can run the tests by running the `cargo test`
commands. Additionnaly, you can also generate a code coverage report with
`cargo tarpaulin --out Html`

# Architecture

This project is separated into multiple separate `crates`. Apart from the
obvious decoupling this provides, it helps with compilation time as we have less
dependencies (You can check the dependency tree with the `cargo tree` command).
This means we can also adapt our models to multiple apps and back ends (re-using
the same core logic for a graphql back end, a rest api, a cli, and a worker
fetching excel sheets or whatever)

Also something to note specifically on the domain representation, I have tried
as much as possible to make invalid states impossible to obtain from another
crate. This means the `core` crate will always act as a "source of truth" for
what is possible and what isn't in our domain. It's good because that means we
don't have to duplicate logic across multiple services and apps, taking the risk
to create invalid state and breaking other services, but adds some overhead and
hassle when interacting with the domain (see the subdivision tests at
`crates/core/office/subdivision.rs::subdivided_available_positions_test`; all
created objects are garanteed to be valid from a business/domain perspective as
long as they are `Ok(_)`). I'd be happy to discuss about the tradeoffs of this
approach in a fast-paced environment.

# Storage

I went with postgresql since I didn't feel the need for an approach other than
relationnal. A simple docker compose configuration is also provided with the
project to set it up so it should be easy locally as well.
