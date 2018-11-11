class Expend < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./etc/brew/expend.rb.in'
  version 'v1.0.0'
  desc "Automate repetitive expenses for Expensify.com"
  homepage "https://github.com/Byron-TW/expend-rs"

  url "https://github.com/Byron-TW/expend-rs/releases/download/v1.0.0/expend-v1.0.0-x86_64-apple-darwin.tar.gz"
  sha256 "4e1dd6919ac530711ab1d48bea7c96d8072d2440e4b6d9bce836ba52d1d8446a"

  def install
    bin.install "expend"
  end
end
