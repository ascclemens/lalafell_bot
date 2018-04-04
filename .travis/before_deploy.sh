set -e

# move to build dir in case we aren't there
cd "$TRAVIS_BUILD_DIR"

# download and extract the send utility
wget https://github.com/jkcclemens/travis_ssh_deploy/releases/download/v0.1.0-alpha.1/linux-x86_64.tar.xz
tar xvf linux-x86_64.tar.xz send

# compress the binary
xz -c9 target/release/lalafell_bot > target/release/lalafell_bot.xz

# decrypt the key for accessing the deploy server
openssl aes-256-cbc -K "$encrypted_e255869f9b16_key" -iv "$encrypted_e255869f9b16_iv" -in .travis/travis_ed25519.enc -out "$HOME/.ssh/id_ed25519" -d
chmod 0600 "$HOME/.ssh/id_ed25519"
