local queries = require("queries");

-- Create some sample data
local id_1 = create_vertex("foo");
local id_2 = create_vertex("foo");
local id_3 = create_vertex("foo");
local id_4 = create_vertex("foo");
local id_5 = create_vertex("foo");
create_edge(id_1, "bar", id_2, 1.0);
create_edge(id_2, "bar", id_3, 1.0);
create_edge(id_3, "bar", id_4, 1.0);
create_edge(id_4, "bar", id_5, 1.0);

function check_vertices(vertices, expected_count, required_vertex_ids)
    assert(table.getn(vertices) >= expected_count);

    for _, vertex in pairs(vertices) do
        if required_vertex_ids[vertex.id] ~= nil then
            assert(vertex.type == "foo")
        end

        required_vertex_ids[vertex.id] = nil
    end

    if table.getn(required_vertex_ids) > 0 then
        error("Not all of the required vertex IDs were found in querying")
    end
end

-- Ensure we can get all of the vertices
local vertices = get_vertices(queries.all_vertices("00000000-0000-0000-0000-000000000000", 10).query)
check_vertices(vertices, 5, {[id_1]=true, [id_2]=true, [id_3]=true, [id_4]=true, [id_5]=true});

-- Ensure we can get a single vertex
local vertices = get_vertices(queries.vertex(id_1).query)
check_vertices(vertices, 1, {[id_1]=true});

-- Ensure we can get a specific set of vertices
local vertices = get_vertices(queries.vertices({id_1, id_2, id_3}).query)
check_vertices(vertices, 3, {[id_1]=true, [id_2]=true, [id_3]=true});

-- Ensure we can do a piped query
local query = queries.vertex(id_1)
    :outbound_edges("bar", queries.json_null, queries.json_null, 1):inbound_vertices(1)
    :outbound_edges(queries.json_null, queries.json_null, queries.json_null, 1):inbound_vertices(1)
    :outbound_edges(queries.json_null, queries.json_null, queries.json_null, 1):inbound_vertices(1)
    :outbound_edges(queries.json_null, queries.json_null, queries.json_null, 1):inbound_vertices(1);
local vertices = get_vertices(query.query);
check_vertices(vertices, 1, {[id_5]=true});
