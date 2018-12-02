class Expend < Formula
  # '---> DO NOT EDIT <--- (this file was generated from ./etc/brew/expend.rb.in'
  version '1.1.0'
  desc "Automate repetitive expenses for Expensify.com"
  homepage "https://github.com/Byron-TW/expend-rs"

  url "https://github.com/Byron-TW/expend-rs/releases/download/1.1.0/expend-1.1.0-x86_64-apple-darwin.tar.gz"
  sha256 "5dd1bd10166570c871979aeeab8168372c972d97db961050c63559c03ad3e5c8"

  def install
    bin.install "expend"
  end
end
