import { content } from "../../data/content";

export function Footer() {
  const { footer } = content;

  return (
    <footer className="border-t border-surface-800/50 py-12 px-6">
      <div className="mx-auto max-w-6xl flex flex-col md:flex-row items-center justify-between gap-6">
        <div>
          <span className="text-lg font-bold text-white">OGBank</span>
          <p className="text-sm text-surface-500 mt-1">{footer.tagline}</p>
        </div>

        <nav className="flex gap-6" aria-label="Footer">
          {footer.links.map((link) => (
            <a
              key={link.label}
              href={link.href}
              className="text-sm text-surface-400 hover:text-white transition-colors nav-link"
            >
              {link.label}
            </a>
          ))}
        </nav>

        <p className="text-xs text-surface-600">{footer.copyright}</p>
      </div>
    </footer>
  );
}
