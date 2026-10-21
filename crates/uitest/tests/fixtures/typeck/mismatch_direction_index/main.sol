function non_callable_base(x: word) {
    let y: bool = x[0];
}

function index_argument(g: function(bool) returns (word), x: word) {
    let y: word = g[x];
}

function index_result(g: function(word) returns (bool), x: word) {
    let y: word = g[x];
}

function index_return(g: function(word) returns (bool), x: word) returns (word) {
    return g[x];
}

function comptime_index_argument(comptime g: function(bool) returns (word), x: word) {
    let y: word = g[x];
}

function comptime_index_result(comptime g: function(word) returns (bool), x: word) {
    let y: word = g[x];
}
