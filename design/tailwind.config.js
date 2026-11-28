// HTM-05: Tailwind build for the design page. The page used the Play CDN with its default theme, so this config adds
// nothing; it only tells the CLI which files to scan for class names (including class names built inside JS strings).
module.exports = {
  content: ["./nexora-v1.html", "./island-activities.js", "./settings-schema.js"],
};
// Rebuild after changing classes in the HTML or JS (tailwindcss 3.4.17 keeps the output stable):
//   cd design && npx tailwindcss@3.4.17 -c tailwind.config.js -i tailwind.input.css -o tailwind.css --minify
// tailwind.css is generated; do not edit it by hand.
