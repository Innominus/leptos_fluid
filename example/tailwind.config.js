/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["*.html", "./src/**/*.rs", "node_modules/preline/dist/*.js"],
  theme: {
    extend: {},
  },
  daisyui: {
    themes: [
      {
        wireframe: {
          ...require("daisyui/src/theming/themes")["wireframe"],
        },
      },
    ],
  },
  plugins: [
    require("@tailwindcss/typography"),
    require("daisyui"),
    require("preline/plugin"),
  ],
};
