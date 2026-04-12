Feature: Web Worker deployment
  As a browser integrator
  I want the WASM binding to support worker-hosted sessions
  So that parsing, validation, and transport stepping stay off the main thread

  Background:
    Given the worker model is v2/specs/07-worker-model.md

  Scenario: Command and event message families
    When the main thread sends document/open or transport/command messages
    Then the worker returns structured responses and batched events when applicable

  Scenario: No synchronous DOM access from core logic
    Given the runtime runs inside a worker
    When transport or sequencer logic executes
    Then it does not require synchronous main-thread browser APIs
