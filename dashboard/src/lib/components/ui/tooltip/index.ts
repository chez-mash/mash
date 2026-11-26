import { Tooltip as TooltipPrimitive } from "bits-ui";
import Tip from "./tip.svelte";
import Content from "./tooltip-content.svelte";

const Root = TooltipPrimitive.Root;
const Trigger = TooltipPrimitive.Trigger;
const Provider = TooltipPrimitive.Provider;

export { Content, Provider, Root, Tip, Trigger };
