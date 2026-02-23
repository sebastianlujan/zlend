export interface SplitResult {
  chars: HTMLSpanElement[];
  words: HTMLSpanElement[];
  revert: () => void;
}

export function splitText(
  element: HTMLElement,
  type: "chars" | "words" | "both" = "chars",
): SplitResult {
  const originalHTML = element.innerHTML;
  const text = element.textContent ?? "";

  // Preserve accessibility
  element.setAttribute("aria-label", text);

  const chars: HTMLSpanElement[] = [];
  const words: HTMLSpanElement[] = [];

  element.innerHTML = "";

  const wordStrings = text.split(/(\s+)/);

  for (const wordStr of wordStrings) {
    if (/^\s+$/.test(wordStr)) {
      element.appendChild(document.createTextNode(wordStr));
      continue;
    }

    const wordSpan = document.createElement("span");
    wordSpan.className = "split-word";
    wordSpan.style.display = "inline-block";
    wordSpan.setAttribute("aria-hidden", "true");
    words.push(wordSpan);

    if (type === "words") {
      wordSpan.textContent = wordStr;
    } else {
      for (const char of wordStr) {
        const charSpan = document.createElement("span");
        charSpan.className = "split-char";
        charSpan.style.display = "inline-block";
        charSpan.setAttribute("aria-hidden", "true");
        charSpan.textContent = char;
        chars.push(charSpan);
        wordSpan.appendChild(charSpan);
      }
    }

    element.appendChild(wordSpan);
  }

  function revert() {
    element.innerHTML = originalHTML;
    element.removeAttribute("aria-label");
  }

  return { chars, words, revert };
}
