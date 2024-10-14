import { ConfigSpec } from "https://deno.land/x/embassyd_sdk@v0.3.4.3.0-alpha1/types.ts";
import { compat, types as T } from "../dependencies.ts";

export const getConfig: T.ExpectedExports.getConfig = async (effects) => {
  const allowUnpruned = (await effects.diskUsage()).total > 800_000_000_000;
  return compat.getConfig({
    "peer-tor-address": {
      name: "Peer Tor Address",
      description: "The Tor address of the peer interface",
      type: "pointer",
      subtype: "package",
      "package-id": "bitcoind",
      target: "tor-address",
      interface: "peer",
    },
    "rpc-tor-address": {
      name: "RPC Tor Address",
      description: "The Tor address of the RPC interface",
      type: "pointer",
      subtype: "package",
      "package-id": "bitcoind",
      target: "tor-address",
      interface: "rpc",
    },
    blkconstr: {
      type: "object",
      name: "Mempool",
      description: "Options about standard policies and mining block templates",
      spec: {
        datacarrier: {
          type: "boolean",
          name: "Datacarrier",
          description: "Relay and mine data carrier transactions",
          default: true,
        },
        datacarriersize: {
          type: "number",
          nullable: false,
          name: "Datacarrier Size",
          description:
            "Maximum size of arbitrary data to relay and mine",
          range: "[0,10000]",
          integral: true,
          units: "bytes",
          default: 42,
        },
        permitbaremultisig: {
          type: "boolean",
          name: "Permit Bare Multisig",
          description: "Relay non-P2SH multisig",
          default: false,
        },
        rejectparasites: {
          type: "boolean",
          name: "Reject Parasites",
          description: "Reject parasite transactions",
          default: true,
        },
        rejecttokens: {
          type: "boolean",
          name: "Reject Tokens",
          description: "Reject tokens transactions (runes)",
          default: false,
        },
        persistmempool: {
          type: "boolean",
          name: "Persist Mempool",
          description: "Save the mempool on shutdown and load on restart.",
          default: true,
        },
        // maxmempool: {
        //   type: "number",
        //   nullable: false,
        //   name: "Max Mempool Size",
        //   description:
        //     "Keep the transaction memory pool below <n> megabytes.",
        //   range: "[1,*)",
        //   integral: true,
        //   units: "MiB",
        //   default: 300,
        // },
        mempoolexpiry: {
          type: "number",
          nullable: false,
          name: "Mempool Expiration",
          description:
            "Do not keep transactions in the mempool longer than <n> hours.",
          range: "[1,*)",
          integral: true,
          units: "Hr",
          default: 336,
        },
        mempoolfullrbf: {
          name: "Enable Full RBF",
          description:
            "Policy for your node to use for relaying and mining unconfirmed transactions.  For details, see https://github.com/bitcoin/bitcoin/blob/master/doc/release-notes/release-notes-24.0.md#notice-of-new-option-for-transaction-replacement-policies",
          type: "boolean",
          default: true,
        },
        minrelaytxfee: {
          type: "number",
          nullable: false,
          name: "Min Transaction Relay Fee",
          description:
            "Fee rates (in BTC/kB) smaller than this are considered zero fee for relaying, mining and transaction creation",
          range: "[0,21000000]",
          integral: false,
          units: "BTC/kvB",
          default: 0.00001,
        },
        bytespersigop: {
          type: "number",
          nullable: false,
          name: "Bytes Per Sigop",
          description:
            "Equivalent bytes per sigop in transactions for relay and mining",
          range: "[0,*)",
          integral: true,
          units: "bytes",
          default: 20,
        },
        bytespersigopstrict: {
          type: "number",
          nullable: false,
          name: "Bytes Per Sigop Strict",
          description:
            "Minimum bytes per sigop in transactions we relay and mine",
          range: "[0,*)",
          integral: true,
          units: "bytes",
          default: 20,
        },
        limitancestorcount: {
          type: "number",
          nullable: false,
          name: "Max Ancestor Count",
          description:
            "Do not accept transactions if number of in-mempool ancestors is <n> or more",
          range: "[0,*)",
          integral: true,
          units: undefined,
          default: 25,
        },
        limitancestorsize: {
          type: "number",
          nullable: false,
          name: "Max Ancestor Size",
          description:
            "Do not accept transactions whose size with all in-mempool ancestors exceeds <n> kilobytes",
          range: "[0,*)",
          integral: true,
          units: "kB",
          default: 101,
        },
        limitdescendantcount: {
          type: "number",
          nullable: false,
          name: "Max descendants count",
          description:
            "Do not accept transactions if any ancestor would have <n> or more in-mempool descendants",
          range: "[0,*)",
          integral: true,
          units: undefined,
          default: 25,
        },
        limitdescendantsize: {
          type: "number",
          nullable: false,
          name: "Max descendants size",
          description:
            "Do not accept transactions if any ancestor would have more than <n> kilobytes of in-mempool descendants",
          range: "[0,*)",
          integral: true,
          units: "kB",
          default: 101,
        },
        permitbarepubkey: {
          type: "boolean",
          name: "Permit Bare Pubkey",
          description: "Relay legacy pubkey outputs",
          default: false,
        },
        maxscriptsize: {
          type: "number",
          nullable: false,
          name: "Max Script Size",
          description:
            "Maximum size of scripts we relay and mine, in bytes",
          range: "[0,*)",
          integral: true,
          units: "Bytes",
          default: 1650,
        },
        datacarriercost: {
          type: "number",
          nullable: false,
          name: "Datacarrier cost",
          description:
            "Treat extra data in transactions as at least N vbytes per actual byte",
          range: "[0,*)",
          integral: true,
          units: undefined,
          default: 1,
        },
        acceptnonstddatacarrier: {
          type: "boolean",
          name: "Accept non standard datacarrier",
          description: "Relay and mine non-OP_RETURN datacarrier injection",
          default: false,
        },
        dustrelayfee: {
          type: "number",
          nullable: false,
          name: "Dust Relay Fee",
          description:
            "Fee rate (in BTC/kvB) used to define dust, the value of an output such that it will cost more than its value in fees at this fee rate to spend it.",
          range: "[0,*)",
          integral: false,
          units: "BTC/kvB",
          default: 0.00003,
        },
      }
    },
    rpc: {
      type: "object",
      name: "RPC Settings",
      description: "RPC configuration options.",
      spec: {
        enable: {
          type: "boolean",
          name: "Enable",
          description: "Allow remote RPC requests.",
          default: true,
        },
        username: {
          type: "string",
          nullable: false,
          name: "Username",
          description: "The username for connecting to Bitcoin over RPC.",
          warning: "You will need to restart all services that depend on Bitcoin.",
          default: "bitcoin",
          masked: true,
          pattern: "^[a-zA-Z0-9_]+$",
          "pattern-description":
            "Must be alphanumeric (can contain underscore).",
        },
        password: {
          type: "string",
          nullable: false,
          name: "RPC Password",
          description: "The password for connecting to Bitcoin over RPC.",
          warning: "You will need to restart all services that depend on Bitcoin.",
          default: {
            charset: "a-z,2-7",
            len: 20,
          },
          pattern: "^[a-zA-Z0-9_]+$",
          "pattern-description":
            "Must be alphanumeric (can contain underscore).",
          copyable: true,
          masked: true,
        },
        advanced: {
          type: "object",
          name: "Advanced",
          description: "Advanced RPC Settings",
          spec: {
            auth: {
              name: "Authorization",
              description:
                "Username and hashed password for JSON-RPC connections. RPC clients connect using the usual http basic authentication.",
              type: "list",
              subtype: "string",
              default: [],
              spec: {
                pattern:
                  "^[a-zA-Z0-9_-]+:([0-9a-fA-F]{2})+\\$([0-9a-fA-F]{2})+$",
                "pattern-description":
                  'Each item must be of the form "<USERNAME>:<SALT>$<HASH>".',
              },
              range: "[0,*)",
            },
            servertimeout: {
              name: "Rpc Server Timeout",
              description:
                "Number of seconds after which an uncompleted RPC call will time out.",
              type: "number",
              nullable: false,
              range: "[5,300]",
              integral: true,
              units: "seconds",
              default: 30,
            },
            threads: {
              name: "Threads",
              description:
                "Set the number of threads for handling RPC calls. You may wish to increase this if you are making lots of calls via an integration.",
              type: "number",
              nullable: false,
              default: 16,
              range: "[1,64]",
              integral: true,
              units: undefined,
            },
            workqueue: {
              name: "Work Queue",
              description:
                "Set the depth of the work queue to service RPC calls. Determines how long the backlog of RPC requests can get before it just rejects new ones.",
              type: "number",
              nullable: false,
              default: 128,
              range: "[8,256]",
              integral: true,
              units: "requests",
            },
          },
        },
      },
    },
    "zmq-enabled": {
      type: "boolean",
      name: "ZeroMQ Enabled",
      description: "Enable the ZeroMQ interface",
      default: true,
    },
    txindex: {
      type: "boolean",
      name: "Transaction Index",
      description: "Enable the Transaction Index (txindex)",
      default: allowUnpruned,
    },
    wallet: {
      type: "object",
      name: "Wallet",
      description: "Wallet Settings",
      spec: {
        enable: {
          name: "Enable Wallet",
          description: "Load the wallet and enable wallet RPC calls.",
          type: "boolean",
          default: true,
        },
        avoidpartialspends: {
          name: "Avoid Partial Spends",
          description:
            "Group outputs by address, selecting all or none, instead of selecting on a per-output basis. This improves privacy at the expense of higher transaction fees.",
          type: "boolean",
          default: true,
        },
        discardfee: {
          name: "Discard Change Tolerance",
          description:
            "The fee rate (in BTC/kB) that indicates your tolerance for discarding change by adding it to the fee.",
          type: "number",
          nullable: false,
          default: 0.0001,
          range: "[0,.01]",
          integral: false,
          units: "BTC/kB",
        },
      },
    },
    advanced: {
      type: "object",
      name: "Advanced",
      description: "Advanced Settings",
      spec: {
        mempool: {
          type: "object",
          name: "Mempool",
          description: "Mempool Settings",
          spec: {
            maxmempool: {
              type: "number",
              nullable: false,
              name: "Max Mempool Size",
              description:
                "Keep the transaction memory pool below <n> megabytes.",
              range: "[1,*)",
              integral: true,
              units: "MiB",
              default: 300,
            },
	  },
        },
        peers: {
          type: "object",
          name: "Peers",
          description: "Peer Connection Settings",
          spec: {
            listen: {
              type: "boolean",
              name: "Make Public",
              description: "Allow other nodes to find your server on the network.",
              default: true,
            },
            onlyconnect: {
              type: "boolean",
              name: "Disable Peer Discovery",
              description: "Only connect to specified peers.",
              default: false,
            },
            onlyonion: {
              type: "boolean",
              name: "Disable Clearnet",
              description: "Only connect to peers over Tor.",
              default: false,
            },
            v2transport: {
              type: "boolean",
              name: "Use V2 P2P Transport Protocol",
              description: "Enable or disable the use of BIP324 V2 P2P transport protocol.",
              default: false,
            },
            addnode: {
              name: "Add Nodes",
              description: "Add addresses of nodes to connect to.",
              type: "list",
              subtype: "object",
              range: "[0,*)",
              default: [],
              spec: {
                spec: {
                  hostname: {
                    type: "string",
                    nullable: false,
                    name: "Hostname",
                    description: "Domain or IP address of bitcoin peer",
                    pattern:
                      "(^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$)|((^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))$)|(^[a-z2-7]{16}\\.onion$)|(^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]$))",
                    "pattern-description":
                      "Must be either a domain name, or an IPv4 or IPv6 address. Do not include protocol scheme (eg 'http://') or port.",
                  },
                  port: {
                    type: "number",
                    nullable: true,
                    name: "Port",
                    description: "Port that peer is listening on for inbound p2p connections",
                    range: "[0,65535]",
                    integral: true,
                  },
                },
              },
            },
          },
        },
        pruning: {
          type: "union",
          name: "Pruning Settings",
          description:
            "Blockchain Pruning Options\nReduce the blockchain size on disk\n",
          warning:
            "Disabling pruning will convert your node into a full archival node. This requires a resync of the entire blockchain, a process that may take several days. Make sure you have enough free disk space or you may fill up your disk.\n",
          tag: {
            id: "mode",
            name: "Pruning Mode",
            description:
              "- Disabled: Disable pruning\n- Automatic: Limit blockchain size on disk to a certain number of megabytes\n",
            "variant-names": {
              disabled: "Disabled",
              automatic: "Automatic",
            },
          },
          variants: {
            disabled: allowUnpruned ? {} : (undefined as unknown as ConfigSpec),
            automatic: {
              size: {
                type: "number",
                nullable: false,
                name: "Max Chain Size",
                description: "Limit of blockchain size on disk.",
                warning:
                  "Increasing this value will require re-syncing your node.",
                default: 550,
                range: "[550,1000000)",
                integral: true,
                units: "MiB",
              },
            },
          },
          default: allowUnpruned ? "disabled" : "automatic",
        },
        dbcache: {
          type: "number",
          nullable: true,
          name: "Database Cache",
          description:
            "How much RAM to allocate for caching the TXO set. Higher values improve syncing performance, but increase your chance of using up all your system's memory or corrupting your database in the event of an ungraceful shutdown. Set this high but comfortably below your system's total RAM during IBD, then turn down to 450 (or leave blank) once the sync completes.",
          warning:
            "WARNING: Increasing this value results in a higher chance of ungraceful shutdowns, which can leave your node unusable if it happens during the initial block download. Use this setting with caution. Be sure to set this back to the default (450 or leave blank) once your node is synced. DO NOT press the STOP button if your dbcache is large. Instead, set this number back to the default, hit save, and wait for bitcoind to restart on its own.",
          range: "(0,*)",
          integral: true,
          units: "MiB",
        },
        blocknotify: {
          type: "string",
          nullable: true,
          name: "Block notify",
          description:
            "Execute command when the best block changes",
          warning: "You will need to restart all services that depend on Bitcoin.",
        },
        templateconstruction: {
          type: "object",
          name: "Template Construction",
          description: "Set limits for block size/weight",
          spec: {
            blockmaxsize: {
              type: "number",
              name: "Max Block Size",
              description: "Maximum block size in bytes",
              integral: true,
              range: "[100000,3985000]",
              default: 3985000,
              nullable: false,
              },
            blockmaxweight: {
              type: "number",
              name: "Max Block Weight",
              description: "Maximum block weight in vBytes",
              integral: true,
              range: "[100000,3985000]",
              default: 3985000,
              nullable: false,
            },
          },
        },
        blockfilters: {
          type: "object",
          name: "Block Filters",
          description: "Settings for storing and serving compact block filters",
          spec: {
            blockfilterindex: {
              type: "boolean",
              name: "Compute Compact Block Filters (BIP158)",
              description:
                "Generate Compact Block Filters during initial sync (IBD) to enable 'getblockfilter' RPC. This is useful if dependent services need block filters to efficiently scan for addresses/transactions etc.",
              default: true,
            },
            peerblockfilters: {
              type: "boolean",
              name: "Serve Compact Block Filters to Peers (BIP157)",
              description:
                "Serve Compact Block Filters as a peer service to other nodes on the network. This is useful if you wish to connect an SPV client to your node to make it efficient to scan transactions without having to download all block data.  'Compute Compact Block Filters (BIP158)' is required.",
              default: false,
            },
          },
        },
        bloomfilters: {
          type: "object",
          name: "Bloom Filters (BIP37)",
          description: "Setting for serving Bloom Filters",
          spec: {
            peerbloomfilters: {
              type: "boolean",
              name: "Serve Bloom Filters to Peers",
              description:
                "Peers have the option of setting filters on each connection they make after the version handshake has completed. Bloom filters are for clients implementing SPV (Simplified Payment Verification) that want to check that block headers  connect together correctly, without needing to verify the full blockchain.  The client must trust that the transactions in the chain are in fact valid.  It is highly recommended AGAINST using for anything except Bisq integration.",
              warning:
                "This is ONLY for use with Bisq integration, please use Block Filters for all other applications.",
              default: false,
            },
          },
        },
      },
    },
  })(effects);
};
