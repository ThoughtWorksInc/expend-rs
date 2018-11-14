class Expend < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./etc/brew/expend.rb.in'
  version '1.0.1'
  desc "Automate repetitive expenses for Expensify.com"
  homepage "https://github.com/Byron-TW/expend-rs"

  url "https://github.com/Byron-TW/expend-rs/releases/download/1.0.1/expend-1.0.1-x86_64-apple-darwin.tar.gz"
  sha256 "a7953295fe51e637d16835a8da5e8e10b9796a3af3fe4b523dc9d169a125b3ce"

  def install
    bin.install "expend"
  end
end
