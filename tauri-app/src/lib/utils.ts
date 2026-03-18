import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { RequestType } from "./types/inspector";

const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type {
  WithElementRef,
  WithoutChild,
  WithoutChildrenOrChild,
} from "bits-ui";

export function createLegitSvelteId(id: number | string, source: RequestType) {
  return `${source}-${id}`;
}

export function formatTimestampString(input: string): string {
  const match = input.match(/^\d{4}-(\d{2})-(\d{2})T(\d{2}:\d{2}:\d{2})/);

  if (!match) {
    throw new Error("Invalid date string format provided.");
  }

  // Parse "03" to integer 3, subtract 1 to get the correct array index (2)
  const monthIndex = parseInt(match[1], 10) - 1;
  const monthName = MONTHS[monthIndex];

  // Using parseInt on the day removes any leading zeros (e.g., "05" becomes "5")
  const day = parseInt(match[2], 10);
  const time = match[3];

  return `${monthName} ${day} ${time}`;
}
