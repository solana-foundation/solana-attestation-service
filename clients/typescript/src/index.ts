import { VERSION, featureFlags } from "@lightprotocol/stateless.js";

// Enable V2 for Light Protocol stateless.js
featureFlags.version = VERSION.V2;

export * from "./generated";
export * from "./pdas";
export * from "./utils";
export * from "./constants";
export * from "./compressed";
