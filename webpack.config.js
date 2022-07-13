const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const path = require("path");

module.exports = {
  entry: "./web/index.ts",
  mode: process.env.NODE_ENV === "development" ? "development" : "production",
  plugins: [
    new WasmPackPlugin({
      crateDirectory: "rust",
      args: "--log-level error",
      outDir: path.resolve(__dirname, "pkg"),
      pluginLogLevel: "error",
    }),
    new HtmlWebpackPlugin({
      template: "web/index.html",
      favicon: "assets/favicon.png",
    }),
  ],
  resolve: {
    extensions: [".ts", ".js"],
  },
  output: {
    clean: true,
    filename: "[name].[contenthash].js",
  },
  performance: {
    hints: false,
  },
  experiments: {
    topLevelAwait: true,
    asyncWebAssembly: true,
  },
  devServer: {
    hot: false,
  },
  watchOptions: {
    aggregateTimeout: 2000,
    ignored: "pkg",
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
          options: {
            presets: ["@babel/preset-typescript"],
          },
        },
      },
      {
        test: /\.(vert|frag)$/,
        exclude: /node_modules/,
        use: {
          loader: "webpack-glsl-loader",
        },
      },
      {
        test: /\.(png|mp3)$/,
        exclude: /node_modules/,
        type: "asset/resource",
      },
    ],
  },
};
