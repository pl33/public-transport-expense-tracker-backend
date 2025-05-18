# SPDX-License-Identifier: MPL-2.0
#   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import pytest

from client.services.Ride_service import *
from client.services.Tag_service import *
from server_fixtures import *


@pytest.fixture
def post_ride(api_config_dict):
    created_ride = routes_ride_post(
        Ride(
            journey_departure="2025-03-01T15:15:00Z",
            location_from="Berlin",
            location_to="Hamburg",
            is_template=False,
        ),
        api_config_dict["read_write"],
    )
    return created_ride


@pytest.fixture
def post_tags(api_config_dict):
    created_tags = []
    for tag in [
        Tag(
            tag_type="integer",
            tag_key="int_tag",
            tag_name="Integer Tag",
        ),
        Tag(
            tag_type="float",
            tag_key="float_tag",
        ),
        Tag(
            tag_type="string",
            tag_key="str_tag",
        ),
        Tag(
            tag_type="date_time",
            tag_key="dt_tag",
        ),
    ]:
        created_tags.append(routes_tag_post(tag, api_config_dict["read_write"]))
    return created_tags


class X(Value):
    type: str
    value: Any


@pytest.fixture
def sample_links():
    return [
        RideTagLink(
            order=1,
            value=Value(type="Integer", value=1),#1,
        ),
        RideTagLink(
            order=2,
            value=Value(type="Integer", value=2),#2,
        ),
        RideTagLink(
            order=2,
            value=Value(type="Float", value=2.0),#2.0,
        ),
        RideTagLink(
            order=3,
            value=Value(type="String", value="3"),#"3",
        ),
        RideTagLink(
            order=4,
            value=Value(type="DateTime", value="2025-05-18T22:17:00Z"),#"2025-05-18T22:17:00Z",
        ),
    ]


def test_list(post_ride, api_config_read):
    rides = routes_ride_tag_list(post_ride.id, api_config_read)
    assert len(rides) == 0


def test_create(api_config_dict, post_ride, post_tags, sample_links):
    created_link = routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0], api_config_dict["read_write"])
    assert created_link.id == 1
    assert created_link.order == sample_links[0].order
    assert created_link.value == sample_links[0].value
    assert created_link.ride_id == post_ride.id
    assert created_link.tag_id == post_tags[0].id

    links = routes_ride_tag_list(post_ride.id, api_config_dict["read"])
    assert len(links) == 1

    assert links[0].link.order == sample_links[0].order
    assert links[0].link.value == sample_links[0].value
    assert links[0].link.ride_id == post_ride.id
    assert links[0].link.tag_id == post_tags[0].id


def test_read(api_config_dict, post_ride, post_tags, sample_links):
    created_link = routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0], api_config_dict["read_write"])
    assert created_link.id == 1
    assert created_link.order == sample_links[0].order
    assert created_link.value == sample_links[0].value
    assert created_link.ride_id == post_ride.id
    assert created_link.tag_id == post_tags[0].id

    read_link = routes_ride_tag_get_by_tag_id(post_ride.id, post_tags[0].id, api_config_dict["read"])
    assert read_link.link.id == 1
    assert read_link.link.order == sample_links[0].order
    assert read_link.link.value == sample_links[0].value
    assert read_link.link.ride_id == post_ride.id
    assert read_link.link.tag_id == post_tags[0].id

    read_link = routes_ride_tag_get_by_link_id(1, api_config_dict["read"])
    assert read_link.link.id == 1
    assert read_link.link.order == sample_links[0].order
    assert read_link.link.value == sample_links[0].value
    assert read_link.link.ride_id == post_ride.id
    assert read_link.link.tag_id == post_tags[0].id


def test_update(api_config_dict, post_ride, post_tags, sample_links):
    created_link = routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0], api_config_dict["read_write"])
    assert created_link.id == 1

    routes_ride_tag_put(1, sample_links[1], api_config_dict["read_write"])

    links = routes_ride_tag_list(post_ride.id, api_config_dict["read"])
    assert len(links) == 1

    assert links[0].link.order == sample_links[1].order
    assert links[0].link.value == sample_links[1].value
    assert links[0].link.ride_id == post_ride.id
    assert links[0].link.tag_id == post_tags[0].id


def test_delete(api_config_dict, post_ride, post_tags, sample_links):
    created_link = routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0],
                                                  api_config_dict["read_write"])
    assert created_link.id == 1
    created_link = routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[1].id, sample_links[3],
                                                  api_config_dict["read_write"])
    assert created_link.id == 2

    routes_ride_tag_delete(1, api_config_dict["read_write"])

    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_get_by_link_id(1, api_config_dict["read"])
    assert exc.value.status_code == 404

    _ = routes_ride_tag_get_by_link_id(2, api_config_dict["read"])

    links = routes_ride_tag_list(post_ride.id, api_config_dict["read"])
    assert len(links) == 1

    assert links[0].link.order == sample_links[3].order
    assert links[0].link.value == sample_links[3].value
    assert links[0].link.ride_id == post_ride.id
    assert links[0].link.tag_id == post_tags[1].id

#####################################################################

def test_list_unauthorized(post_ride, api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_list(post_ride.id, api_config_unauthorized)
    assert exc.value.status_code == 401


def test_read_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_get_by_link_id(1, api_config_unauthorized)
    assert exc.value.status_code == 401
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_get_by_tag_id(1, 1, api_config_unauthorized)
    assert exc.value.status_code == 401


def test_create_unauthorized(api_config_unauthorized, post_ride, post_tags, sample_links):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_update_unauthorized(api_config_unauthorized, sample_links):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_put(1, sample_links[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_delete_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_delete(1, api_config_unauthorized)
    assert exc.value.status_code == 401

#####################################################################

def test_create_no_rights(api_config_read, post_ride, post_tags, sample_links):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0], api_config_read)
    assert exc.value.status_code == 401


def test_update_no_rights(api_config_read, sample_links):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_put(1, sample_links[0], api_config_read)
    assert exc.value.status_code == 401


def test_delete_no_rights(api_config_read):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_delete(1, api_config_read)
    assert exc.value.status_code == 401

#####################################################################

@pytest.fixture
def wrong_owner(api_config_dict, post_ride, post_tags, sample_links):
    link = routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0], api_config_dict["read_write"])
    assert link.id == 1
    return api_config_dict["read_write_2"]


def test_read_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_get_by_link_id(1, wrong_owner)
    assert exc.value.status_code == 404


def test_update_wrong_owner(wrong_owner, sample_links):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_put(1, sample_links[0], wrong_owner)
    assert exc.value.status_code == 404


def test_delete_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_delete(1, wrong_owner)
    assert exc.value.status_code == 404

#####################################################################

@pytest.fixture
def no_ride(api_config_dict, post_ride, post_tags):
    routes_ride_delete(post_ride.id, api_config_dict["read_write"])
    return post_ride


def test_list_no_ride(no_ride, api_config_read):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_list(no_ride.id, api_config_read)
    assert exc.value.status_code == 404


def test_create_no_ride(no_ride, post_tags, sample_links, api_config_dict):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_post_by_tag_id(no_ride.id, post_tags[0].id, sample_links[0], api_config_dict["read_write"])
    assert exc.value.status_code == 404

#####################################################################

def test_list_wrong_ride_owner(post_ride, api_config_dict):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_list(post_ride.id, api_config_dict["read_2"])
    assert exc.value.status_code == 404


def test_create_wrong_ride_owner(post_ride, post_tags, sample_links, api_config_dict):
    with pytest.raises(HTTPException) as exc:
        routes_ride_tag_post_by_tag_id(post_ride.id, post_tags[0].id, sample_links[0], api_config_dict["read_write_2"])
    assert exc.value.status_code == 404
