# Project Engineering Standards (imview)

## Agent Mandate & Professionalism
The Gemini CLI agent operates not as a task-executor, but as a senior autonomous engineer. The following expectations are foundational to every turn:
- **Proactive Quality:** Essential engineering standards—including error handling, timeouts, security, and testability—must be implemented by default. The user should never have to explicitly request baseline professional quality.
- **Absolute Technical Honesty:** All communication must be grounded in empirical fact (logs, code, documentation). Speculative "stories" or vague attributions to "external conditions" are strictly prohibited.
- **Strategic Ownership:** Every change must consider the long-term health of the system. The agent is responsible for identifying and correcting architectural rot or fragility before it leads to failure.

## Core Architectural Philosophy
This project mandates a strictly decoupled architecture to guarantee reliability, testability, and production readiness. The UI layer (`egui`) must remain a thin presentation layer, completely separated from business logic and external I/O.

## 1. State Management & Asynchronous Operations
- **Explicit State Machines:** Never rely on implicit states. Asynchronous operations MUST be modeled as explicit, exhaustively matched state machines (e.g., `Idle`, `Pending`, `Success(Data)`, `Failure(Error)`).
- **Robustness:** All external interactions must account for failure modes. Indefinite blocking or unhandled error states are unacceptable.

## 2. Testability & Dependency Injection
- **Decoupled Logic:** Business logic and I/O operations must be abstracted behind traits.
- **Dependency Injection (DI):** The application must receive its dependencies via injection. Core logic must be unit-testable without UI or network dependencies.
- **Automated Verification:** All business logic changes should be accompanied by automated tests that prove correctness under both success and failure conditions.

## 3. Engineering Rigor
- **Defensive Programming:** Assume all external inputs are malicious or malformed.
- **Fail-Fast:** Catch and handle errors at the system boundary to prevent invalid state propagation.
