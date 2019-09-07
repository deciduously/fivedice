const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
  entry: "./src/client/bootstrap.js",
  output: {
    path: path.resolve(__dirname, "docs"),
    filename: "bootstrap.js"
  },
  mode: "development",
  plugins: [new CopyWebpackPlugin(["./src/client/index.html"])]
};
