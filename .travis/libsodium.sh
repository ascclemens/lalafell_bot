#!/usr/bin/env sh

# The path to install libsodium at
SODIUM_PATH=$HOME/libsodium

# If the cache didn't restore libsodium, build it
if [ ! -d "$SODIUM_PATH" ]; then

  # Clone the libsodium repo
  git clone git://github.com/jedisct1/libsodium.git

  # Change directory
  cd libsodium

  # Create configure
  ./autogen.sh

  # Configure libsodium to be built into $HOME/libsodium
  ./configure --prefix=$SODIUM_PATH

  # Make and install it
  make install

fi

# Tell pkg-config where to find it
export PKG_CONFIG_PATH=$SODIUM_PATH/lib/pkgconfig
