---
name: Bug report
about: Create a report to help us improve

---

## :stop_sign: Do you want to ask a question ?

Please head to the [Discord chat](https://discord.gg/<id>)

## Prerequisites

Please answer the following questions for yourself before submitting an issue.

- [ ] I am running the latest version
- [ ] I checked the documentation and found no answer
- [ ] I checked to make sure that this issue has not already been filed
- [ ] I'm reporting the issue to the correct repository

## Current Behavior

<!-- What is the current behavior? -->

## Expected Behavior

<!-- Please describe the behavior you are expecting -->

## Failure Information

<!-- Please help provide information about the failure if this is a bug. If it is not a bug, please remove the rest of this template. -->

### Steps to Reproduce

Please provide detailed steps for reproducing the issue.

1. step 1
2. step 2
3. you get it...

### Reproduce code snippet
<!-- You can use codesandbox.io to create a reproduce code.
     Just fork it: https://codesandbox.io/embed/<id>   -->

```rust
let broker = ServiceBroker::new(ServiceBrokerOptions {
    logger: ConsoleLogger,
    transporter: "NATS".to_string(),
});

broker.create_service(Service {
    name: "test".to_string(),
    actions: vec![
        Action {
            name: "empty".to_string(),
            handler: |ctx| {
                "Hello".to_string()
            }
        }
    ]
});
broker.start();
```

### Context

Please provide any relevant information about your setup. This is important in case the issue is not reproducible except for under certain conditions.

- Penoos version:
- Rust version:
- Operating System:

### Failure Logs

```

```
