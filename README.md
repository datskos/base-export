## Block exporter

Goal: in Nov 2023 we wanted to spin up an `op-reth` node for base-mainnet.
The only problem was syncing the node from genesis was going to take a significant amount of time.

Since we already had a synced `op-geth` node we wrote a tool to export blocks in RLP-encoded format
and then import them into `op-reth`

### Usage

```
cargo build --release
mkdir data
```

To export blocks 100k at a time, you can create an `export.sh` script like below to export up to block 13.2M (adjust as
needed). You will of course need an already synced node.

```shell
## export.sh
set -e
set -x
for ((i=1; i<=13200000; i+=100000)); do
    start=$i
    end=$((i + 99999))
    echo "Exporting" $start " - " $end
    ./target/release/base-export blocks -r '<BASE_RPC>' -p data/blocks_$end --start $start --end $end
done
```

Due to regression in the reth alpha.22 import pipeline that broke base imports, I've included a patch that you can
apply.

```
## in reth repo, assuming you've made it a sibling to base-export/
git checkout v0.1.0-alpha.22
git apply ../base-export/alpha.22-import.patch

cargo build --release --features=optimism --bin=op-reth
```

To import into a fresh `op-reth` this is how I did it (you'll need to clone `reth` and build `op-reth`)

```shell
## import.sh (in your reth directory, compiled as op-reth)
set -e
set -x
for ((i=1; i<=13200000; i+=100000)); do
    start=$i
    end=$((i + 99999))
    echo "Importing" $start " - " $end
    ./target/release/op-reth import --chain base ../base-export/data/blocks_$end
done
```

Warning: I did this almost 6mo ago so no guarantees. Feel free to reach out if you encounter any issues or bugs.

## Disclaimer

*This code is being provided as is. No guarantee, representation or warranty is being made, express or implied, as to
the safety or correctness of the code. It has not been audited and as such there can be no assurance it will work as
intended, and users may experience delays, failures, errors, omissions or loss of transmitted information. Nothing in
this repo should be construed as investment advice or legal advice for any particular facts or circumstances and is not
meant to replace competent counsel. It is strongly advised for you to contact a reputable attorney in your jurisdiction
for any questions or concerns with respect thereto. Author is not liable for any use of the foregoing, and users should
proceed with caution and use at their own risk..*