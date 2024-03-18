class RustPGDataDiffClientBin < Formula
  version '0.1.8'
  desc "Rust client for comparing two PostgreSQL databases"
  homepage "https://github.com/pavlospt/rust-pgdatadiff"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/pavlospt/rust-pgdatadiff-client/releases/download/#{version}/rust-pgdatadiff-client-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "32754b4173ac87a7bfffd436d601a49362676eb1841ab33440f2f49c002c8967"
    else
      url "https://github.com/pavlospt/rust-pgdatadiff-client/releases/download/#{version}/rust-pgdatadiff-client-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "32754b4173ac87a7bfffd436d601a49362676eb1841ab33440f2f49c002c8967"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/pavlospt/rust-pgdatadiff-client/releases/download/#{version}/rust-pgdatadiff-client-#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "c76080aa807a339b44139885d77d15ad60ab8cdd2c2fdaf345d0985625bc0f97"
    else
      url "https://github.com/pavlospt/rust-pgdatadiff-client/releases/download/#{version}/rust-pgdatadiff-client-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "c76080aa807a339b44139885d77d15ad60ab8cdd2c2fdaf345d0985625bc0f97"
    end
  end

  conflicts_with "rust-pgdatadiff-client"

  def install
    bin.install "rust-pgdatadiff-client"
  end
end
