import { Dialog as DialogPrimitive } from "bits-ui";
import Content from "./dialog-content.svelte";
import Overlay from "./dialog-overlay.svelte";

const Root = DialogPrimitive.Root;
const Title = DialogPrimitive.Title;
const Description = DialogPrimitive.Description;
const Close = DialogPrimitive.Close;
const Trigger = DialogPrimitive.Trigger;

export { Close, Content, Description, Overlay, Root, Title, Trigger };
