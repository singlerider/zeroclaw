// Language switcher injected into mdBook's menu bar.
//
// Detects the current locale from the URL by finding a path segment that
// matches one of LOCALES, then renders a globe-icon dropdown linking to the
// same page in every other locale.
//
// Mirror the LOCALES list in .github/workflows/docs-deploy.yml (env.LOCALES)
// and the po/ directory. All three must stay in sync.
(function () {
  const LOCALES = [
    { code: "en", label: "English" },
    { code: "ja", label: "日本語" },
  ];

  const pathSegments = window.location.pathname.split("/");
  const localeIndex = pathSegments.findIndex((seg) =>
    LOCALES.some((l) => l.code === seg)
  );
  if (localeIndex < 0) return;
  const currentLocale = pathSegments[localeIndex];

  const urlForLocale = (code) => {
    const next = pathSegments.slice();
    next[localeIndex] = code;
    return next.join("/");
  };

  const menuRight = document.querySelector(".menu-bar .right-buttons");
  if (!menuRight) return;

  const button = document.createElement("button");
  button.id = "language-toggle";
  button.className = "icon-button";
  button.type = "button";
  button.title = "Change language";
  button.setAttribute("aria-label", "Change language");
  button.setAttribute("aria-haspopup", "true");
  button.setAttribute("aria-expanded", "false");
  button.setAttribute("aria-controls", "language-list");
  // Inline SVG globe matches mdBook 0.5.x's icon pattern (.fa-svg wrapper,
  // currentColor stroke so it picks up the theme).
  button.innerHTML =
    '<span class="fa-svg">' +
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" ' +
    'fill="none" stroke="currentColor" stroke-width="2" ' +
    'stroke-linecap="round" stroke-linejoin="round">' +
    '<circle cx="12" cy="12" r="10"/>' +
    '<line x1="2" y1="12" x2="22" y2="12"/>' +
    '<path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 ' +
    '15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>' +
    "</svg>" +
    "</span>";

  const list = document.createElement("ul");
  list.id = "language-list";
  list.className = "theme-popup";
  list.setAttribute("aria-label", "Languages");
  list.setAttribute("role", "menu");
  list.style.display = "none";

  for (const { code, label } of LOCALES) {
    const li = document.createElement("li");
    li.setAttribute("role", "none");
    if (code === currentLocale) li.classList.add("theme-selected");
    const item = document.createElement("button");
    item.setAttribute("role", "menuitem");
    item.className = "theme";
    const link = document.createElement("a");
    link.id = code;
    link.textContent = label;
    link.href = urlForLocale(code);
    item.appendChild(link);
    li.appendChild(item);
    list.appendChild(li);
  }

  button.addEventListener("click", (event) => {
    event.stopPropagation();
    const open = list.style.display === "block";
    list.style.display = open ? "none" : "block";
    button.setAttribute("aria-expanded", String(!open));
  });
  document.addEventListener("click", (event) => {
    if (!list.contains(event.target) && event.target !== button) {
      list.style.display = "none";
      button.setAttribute("aria-expanded", "false");
    }
  });

  menuRight.prepend(list);
  menuRight.prepend(button);
})();
