# Round Packet Schema Contract

**Feature**: 076-recursive-stage-refinement
**Version**: 1.0
**Date**: 2026-06-07

## Overview

Every refinement round produces exactly one compact structured round packet. The packet is emitted as a `TraceEventType::RefinementRoundCompleted` event in the trace store and surfaced through `boundline inspect`. Packets reference artifacts by trace identifier rather than copying full content inline.

## JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "RoundPacket",
  "type": "object",
  "required": [
    "schema_version",
    "profile",
    "stage",
    "round",
    "candidate_ref",
    "findings",
    "requested_deltas",
    "applied_deltas",
    "critic_confidence",
    "effective_confidence",
    "confidence_adjustment_reason",
    "stop_reason"
  ],
  "properties": {
    "schema_version": {
      "type": "string",
      "description": "Schema version of the round packet format. Current: \"1.0\".",
      "pattern": "^\\d+\\.\\d+$"
    },
    "profile": {
      "type": "string",
      "description": "Refinement profile name. Current: \"plan_refinement\".",
      "minLength": 1
    },
    "stage": {
      "type": "string",
      "description": "Stage this round belongs to. Current: \"plan\".",
      "minLength": 1
    },
    "round": {
      "type": "integer",
      "description": "1-based round number within the refinement loop.",
      "minimum": 1
    },
    "candidate_ref": {
      "type": "string",
      "description": "Trace artifact reference to the plan candidate (e.g., \"trace://plan-candidate-2\"). Must not contain inline artifact content.",
      "pattern": "^trace://[a-z][a-z0-9_-]*(-\\d+)?$"
    },
    "findings": {
      "type": "array",
      "description": "Finding IDs from this round. References existing Finding entities.",
      "items": {
        "type": "string",
        "minLength": 1
      }
    },
    "requested_deltas": {
      "type": "array",
      "description": "Revision deltas the critic requested for this round.",
      "items": { "$ref": "#/definitions/RevisionDelta" }
    },
    "applied_deltas": {
      "type": "array",
      "description": "Revision deltas the planner applied for this round.",
      "items": { "$ref": "#/definitions/RevisionDelta" }
    },
    "critic_confidence": {
      "$ref": "#/definitions/Confidence",
      "description": "Confidence level proposed by the critic."
    },
    "effective_confidence": {
      "$ref": "#/definitions/Confidence",
      "description": "Confidence level validated by the runtime. Must not exceed critic_confidence. Must not be \"high\" when blocking findings are unresolved."
    },
    "confidence_adjustment_reason": {
      "oneOf": [
        { "type": "null" },
        { "$ref": "#/definitions/ConfidenceAdjustment" }
      ],
      "description": "Reason for adjustment when critic_confidence and effective_confidence differ. Null when they match."
    },
    "stop_reason": {
      "oneOf": [
        { "type": "null" },
        { "$ref": "#/definitions/StopReason" }
      ],
      "description": "Reason the loop stopped. Null if the loop continues to the next round."
    }
  },
  "definitions": {
    "Confidence": {
      "type": "string",
      "enum": ["insufficient", "low", "sufficient", "high"]
    },
    "ConfidenceAdjustment": {
      "type": "string",
      "enum": [
        "blockers_unresolved",
        "high_severity_findings",
        "multiple_medium_findings"
      ]
    },
    "StopReason": {
      "type": "string",
      "enum": [
        "no_material_delta",
        "round_limit_exhausted",
        "time_limit_exhausted",
        "empty_candidate",
        "unresolved_blocker",
        "provider_failure",
        "malformed_packet",
        "invalid_delta",
        "invalid_configuration"
      ]
    },
    "RevisionDelta": {
      "type": "object",
      "required": ["artifact_ref", "kind", "target", "description", "provenance"],
      "properties": {
        "artifact_ref": {
          "type": "string",
          "description": "Trace artifact reference.",
          "pattern": "^trace://[a-z][a-z0-9_-]*(-\\d+)?$"
        },
        "kind": {
          "type": "string",
          "enum": [
            "add_task",
            "remove_task",
            "reorder_task",
            "update_dependency",
            "update_scope",
            "update_validation",
            "update_risk",
            "update_blocker"
          ]
        },
        "target": {
          "type": "string",
          "description": "Specific element being changed (task ID, section, dependency edge).",
          "minLength": 1
        },
        "description": {
          "type": "string",
          "description": "Human-readable description of the change.",
          "minLength": 1
        },
        "provenance": {
          "type": "string",
          "description": "Finding ID that motivated this delta.",
          "minLength": 1
        }
      }
    }
  }
}
```

## Contract Tests

The following contract tests must pass:

1. **Schema completeness**: A valid round packet with all required fields deserializes without error.
2. **No inline content**: A round packet where `candidate_ref` contains a full plan text (not a `trace://` reference) must be rejected.
3. **Confidence invariant**: A round packet where `effective_confidence` is `"high"` and `findings` contains a blocking finding ID must be rejected.
4. **Confidence downgrade only**: A round packet where `effective_confidence` exceeds `critic_confidence` must be rejected.
5. **Stop reason vocabulary**: A round packet with an unrecognized `stop_reason` value must be rejected.
6. **Round numbering**: A round packet with `round = 0` must be rejected.
7. **Delta validity**: A delta whose `artifact_ref` does not match `^trace://` must be rejected.
8. **Null stop_reason semantics**: A round packet with `stop_reason: null` must not have a `finalized` or `incomplete` outcome associated with it; the loop is still running.
