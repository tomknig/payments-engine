# Payments Engine

## Assumptions

### Amount of Money

The [domain model for money](./src/models/money.rs) is designed to have a precision of four decimals. I decided to go with a `u64` for the underlying datatype for storing actual monetary data. It therefore uses `10^4-1 ~= 14 bits` for the fractional part and the remaining `64 - 14 = 50 bits` for the integer part. I am therefore assuming that no single transaction, nor total account balance ever exceeds `2^50 ~= 10^15`, which I think is a fair assumption to make for this toy engine.

### Dispute Management

1. I've decided to accept disputes only when they are targetting deposit transactions. That means, a withdrawal transaction can not be disputed by a client. I came to this assumption from common sense as clients probably only dispute when money is debited from their payment method and not when money is credited to it. Additionally, it is stated as a requirement that "the clients available funds should decrease by the amount disputed" which would be incompatible with disputing withdrawals. In the real world outside of this toy engine, disputing withdrawals e.g. when accounts have been hijacked, is most likely a thing though.
1. Transactions can be disputed more than once, but only if they were resolved. I decided to keep transactions in the pool for disputable transactions even after resolution, so that they can potentially be disputed multiple times. A chargeback is naturally final though, because it freezes the account that has issued the original transaction and dispute.

### Input

While handling some invalid input formats, I generally assume `transaction.csv` files to follow the expected column layout, and:

- client IDs to be valid u16 values
- transaction IDs to be valid u32 values
- amount has a maximum of four fractional digits

Other inputs are not accepted by the engine and skipped.

## AI Usage

During development of this payments engine, I've made use of the following AI tools and practices:

### Edit Predictions

I've developed this application using the [Zed Editor](https://zed.dev/) with [edit preditions](https://zed.dev/docs/ai/edit-prediction) enabled and configured to use [the Zeta model](https://huggingface.co/zed-industries/zeta). Occasionally, edit predictions were a hit and saved me time typing. A concrete example from the [account model](./src/models/account.ts):

Started to type:

```rs
impl Account {
    // [...]
    pub fn total
```

Predicted as, and completed to:

```rs
impl Account {
    // [...]
    pub fn total(&self) -> Money {
        self.available + self.held
    }
```

### ChatGPT

Chatting with LLMs has become an integral part of my workflow, and I've used it for this project, too. While I do find myself referring to concrete crate-level documentation directly to the original source, i.e., [crates.io](https://crates.io), I do refer to ChatGPT (with `GPT-5.2` at time of writing) for research that would have been targeted to StackOverflow years ago. Concrete examples follow here:

#### Deciding for variable names

Naming is hard, I try to let the computer do the hard work:

Prompt:
```md
I want to create a datastructure to manage transactions alongside with the resulting accounts (resulting from ordered application of the transactions). Would a `ledger` accurately describe this datastructure, or am I off?
```

Response
```md
[...]
You are not off at all —
Ledger is a precise and semantically correct name for:

A data structure that stores ordered transactions and produces resulting accounts from applying them.

If anything, it’s the most canonical name for that concept.
[...]
```

#### Refactor rushed test cases

During development of the [domain model for money](./src/models/money.rs), I lazily used one test for multiple assertions that would fail for different reasons. Before commiting this, I had ChatGPT refactor this into four separate unit tests.

``````md
Make this four separate unit tests:
```rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let mut money = Money { value: 123_0000 };
        assert_eq!(money.to_string(), "123.0000");

        money = Money { value: 123_4000 };
        assert_eq!(money.to_string(), "123.4000");

        money = Money { value: 123_0007 };
        assert_eq!(money.to_string(), "123.0007");

        money = Money { value: 123_4567 };
        assert_eq!(money.to_string(), "123.4567");
    }
}
```
``````

### Agentic Coding

While I actually do make use of Codex-/Claude Code-like agentic development in my normal day-to-day, I proactively decided against using it for this task: I did not use agentic code editing in any capacity for this project, as it would somewhat defeat the purpose of it.

### Documentation

The documentation, including this README file, has been written by hand.
