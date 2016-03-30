initSidebarItems({"enum":[["ChannelKind","The kind of the channel, i.e. a strongly-typed description of _what_ the channel can do. Used both for locating channels (e.g. \"I need a clock\" or \"I need something that can provide pictures\") and for determining the data structure that these channel can provide or consume."]],"struct":[["Channel","An channel represents a single place where data can enter or leave a device. Note that channels support either a single kind of getter or a single kind of setter. Devices that support both getters or setters, or several kinds of getters, or several kinds of setters, are represented as services containing several channels."],["Getter","A getter operation available on a channel."],["Service","Metadata on a service. A service is a device or collection of devices that may offer services. The FoxBox itself is a service offering services such as a clock, communicating with the user through her smart devices, etc."],["Setter","An setter operation available on an channel."]],"trait":[["IOMechanism","The communication mechanism used by the channel."]]});