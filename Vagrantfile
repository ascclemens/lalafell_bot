# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/trusty64"

  config.vm.provider "virtualbox" do |vb|
    vb.cpus = 6
    vb.memory = 4072
  end

  config.vm.provision "shell" do |s|
    s.privileged = false
    s.inline = <<-SHELL
      sudo apt-get update
      sudo apt-get install -y pkg-config git cmake libopus-dev libssl-dev libpq-dev

      # Clone the libsodium repo
      git clone -b stable git://github.com/jedisct1/libsodium.git $HOME/libsodium_repo
      # Change directory
      cd $HOME/libsodium_repo
      # Create configure
      ./autogen.sh
      # Configure libsodium to be built into $HOME/libsodium
      ./configure --prefix=$HOME/libsodium
      # Make and install it
      make install

      curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

      echo "export PKG_CONFIG_PATH=$HOME/libsodium/lib/pkgconfig" >> $HOME/.profile
      echo "alias build=\"CARGO_TARGET_DIR=./target cargo build --manifest-path /vagrant/Cargo.toml\"" >> $HOME/.profile
    SHELL
  end
end
