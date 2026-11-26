import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
import CheckboxItem from "./dropdown-menu-checkbox-item.svelte";
import ContentStatic from "./dropdown-menu-content-static.svelte";
import Item from "./dropdown-menu-item.svelte";
import RadioItem from "./dropdown-menu-radio-item.svelte";

const Root = DropdownMenuPrimitive.Root;
const Trigger = DropdownMenuPrimitive.Trigger;
const RadioGroup = DropdownMenuPrimitive.RadioGroup;

export { CheckboxItem, ContentStatic, Item, RadioGroup, RadioItem, Root, Trigger };
