# SPDX-License-Identifier: MPL-2.0
#   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import pytest

from client.services.Tag_service import *
from server_fixtures import *


@pytest.fixture
def sample_tags():
    return [
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
        Tag(
            tag_type="enum",
            tag_key="enum_tag",
        ),
        Tag(
            tag_type="asdf",
            tag_key="invalid_tag",
        ),
    ]


def test_list(api_config_read):
    rides = routes_tag_list(api_config_read)
    assert len(rides) == 0


def test_create(api_config_dict, sample_tags):
    created_tag = routes_tag_post(sample_tags[0], api_config_dict["read_write"])
    assert created_tag.id == 1
    assert created_tag.tag_type == sample_tags[0].tag_type
    assert created_tag.tag_key == sample_tags[0].tag_key
    assert created_tag.tag_name == sample_tags[0].tag_name
    assert created_tag.tag_display_name == sample_tags[0].tag_name

    tags = routes_tag_list(api_config_dict["read"])
    assert len(tags) == 1

    assert tags[0].id == 1
    assert tags[0].tag_type == sample_tags[0].tag_type
    assert tags[0].tag_key == sample_tags[0].tag_key
    assert tags[0].tag_name == sample_tags[0].tag_name
    assert tags[0].tag_display_name == sample_tags[0].tag_name


def test_read(api_config_dict, sample_tags):
    created_tag = routes_tag_post(sample_tags[0], api_config_dict["read_write"])
    assert created_tag.id == 1
    assert created_tag.tag_type == sample_tags[0].tag_type
    assert created_tag.tag_key == sample_tags[0].tag_key
    assert created_tag.tag_name == sample_tags[0].tag_name
    assert created_tag.tag_display_name == sample_tags[0].tag_name

    read_tag = routes_tag_get(1, api_config_dict["read"])
    assert read_tag.id == 1
    assert read_tag.tag_type == sample_tags[0].tag_type
    assert read_tag.tag_key == sample_tags[0].tag_key
    assert read_tag.tag_name == sample_tags[0].tag_name
    assert read_tag.tag_display_name == sample_tags[0].tag_name


def test_update(api_config_dict, sample_tags):
    created_tag = routes_tag_post(sample_tags[0], api_config_dict["read_write"])
    assert created_tag.id == 1

    routes_tag_put(1, sample_tags[1], api_config_dict["read_write"])

    tags = routes_tag_list(api_config_dict["read"])
    assert len(tags) == 1

    assert tags[0].id == 1
    assert tags[0].tag_type == sample_tags[1].tag_type
    assert tags[0].tag_key == sample_tags[1].tag_key
    assert tags[0].tag_name == sample_tags[1].tag_name
    assert tags[0].tag_display_name == sample_tags[1].tag_key


def test_delete(api_config_dict, sample_tags):
    created_tag = routes_tag_post(sample_tags[0], api_config_dict["read_write"])
    assert created_tag.id == 1
    created_tag = routes_tag_post(sample_tags[1], api_config_dict["read_write"])
    assert created_tag.id == 2

    routes_tag_delete(1, api_config_dict["read_write"])

    with pytest.raises(HTTPException) as exc:
        routes_tag_get(1, api_config_dict["read"])
    assert exc.value.status_code == 404

    _ = routes_tag_get(2, api_config_dict["read"])

    tags = routes_tag_list(api_config_dict["read"])
    assert len(tags) == 1

    assert tags[0].id == 2
    assert tags[0].tag_type == sample_tags[1].tag_type
    assert tags[0].tag_key == sample_tags[1].tag_key
    assert tags[0].tag_name == sample_tags[1].tag_name
    assert tags[0].tag_display_name == sample_tags[1].tag_key

#####################################################################

def test_list_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_tag_list(api_config_unauthorized)
    assert exc.value.status_code == 401


def test_read_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_tag_get(1, api_config_unauthorized)
    assert exc.value.status_code == 401


def test_create_unauthorized(api_config_unauthorized, sample_tags):
    with pytest.raises(HTTPException) as exc:
        routes_tag_post(sample_tags[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_update_unauthorized(api_config_unauthorized, sample_tags):
    with pytest.raises(HTTPException) as exc:
        routes_tag_put(1, sample_tags[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_delete_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_tag_delete(1, api_config_unauthorized)
    assert exc.value.status_code == 401

#####################################################################

def test_create_no_rights(api_config_read, sample_tags):
    with pytest.raises(HTTPException) as exc:
        routes_tag_post(sample_tags[0], api_config_read)
    assert exc.value.status_code == 401


def test_update_no_rights(api_config_read, sample_tags):
    with pytest.raises(HTTPException) as exc:
        routes_tag_put(1, sample_tags[0], api_config_read)
    assert exc.value.status_code == 401


def test_delete_no_rights(api_config_read):
    with pytest.raises(HTTPException) as exc:
        routes_tag_delete(1, api_config_read)
    assert exc.value.status_code == 401

#####################################################################

@pytest.fixture
def wrong_owner(api_config_dict, sample_tags):
    _ = routes_tag_post(sample_tags[0], api_config_dict["read_write"])
    return api_config_dict["read_write_2"]


def test_read_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_tag_get(1, wrong_owner)
    assert exc.value.status_code == 404


def test_update_wrong_owner(wrong_owner, sample_tags):
    with pytest.raises(HTTPException) as exc:
        routes_tag_put(1, sample_tags[0], wrong_owner)
    assert exc.value.status_code == 404


def test_delete_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_tag_delete(1, wrong_owner)
    assert exc.value.status_code == 404

#####################################################################

def test_create_invalid_tag(api_config_dict, sample_tags):
    with pytest.raises(HTTPException) as exc:
        routes_tag_post(sample_tags[5], api_config_dict["read_write"])
    assert exc.value.status_code == 400
