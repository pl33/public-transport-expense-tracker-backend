# SPDX-License-Identifier: MPL-2.0
#   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import pytest

from client.services.Tag_service import *
from server_fixtures import *


@pytest.fixture
def post_tag(api_config_dict):
    created_tag = routes_tag_post(
        Tag(
            tag_type="enum",
            tag_key="enum_tag",
        ),
        api_config_dict["read_write"],
    )
    return created_tag


@pytest.fixture
def sample_options():
    return [
        TagOption(
            order=1,
            value="Option 1",
        ),
        TagOption(
            order=2,
            value="Option 2",
            name="opt2",
        ),
        TagOption(
            order=3,
            value="Option 3",
        ),
    ]


def test_list(post_tag, api_config_read):
    rides = routes_tag_option_list(post_tag.id, api_config_read)
    assert len(rides) == 0


def test_create(api_config_dict, post_tag, sample_options):
    created_option = routes_tag_option_post(post_tag.id, sample_options[0], api_config_dict["read_write"])
    assert created_option.id == 1
    assert created_option.order == sample_options[0].order
    assert created_option.value == sample_options[0].value
    assert created_option.display_name == sample_options[0].value

    options = routes_tag_option_list(post_tag.id, api_config_dict["read"])
    assert len(options) == 1

    assert options[0].id == 1
    assert options[0].order == sample_options[0].order
    assert options[0].value == sample_options[0].value
    assert options[0].display_name == sample_options[0].value


def test_read(api_config_dict, post_tag, sample_options):
    created_option = routes_tag_option_post(post_tag.id, sample_options[0], api_config_dict["read_write"])
    assert created_option.id == 1
    assert created_option.order == sample_options[0].order
    assert created_option.value == sample_options[0].value
    assert created_option.display_name == sample_options[0].value

    read_option = routes_tag_option_get(1, api_config_dict["read"])
    assert read_option.id == 1
    assert read_option.order == sample_options[0].order
    assert read_option.value == sample_options[0].value
    assert read_option.display_name == sample_options[0].value


def test_update(api_config_dict, post_tag, sample_options):
    created_option = routes_tag_option_post(post_tag.id, sample_options[0], api_config_dict["read_write"])
    assert created_option.id == 1

    routes_tag_option_put(1, sample_options[1], api_config_dict["read_write"])

    options = routes_tag_option_list(post_tag.id, api_config_dict["read"])
    assert len(options) == 1

    assert options[0].id == 1
    assert options[0].order == sample_options[1].order
    assert options[0].value == sample_options[1].value
    assert options[0].name == sample_options[1].name
    assert options[0].display_name == sample_options[1].name


def test_delete(api_config_dict, post_tag, sample_options):
    created_option = routes_tag_option_post(post_tag.id, sample_options[0], api_config_dict["read_write"])
    assert created_option.id == 1
    created_option = routes_tag_option_post(post_tag.id, sample_options[1], api_config_dict["read_write"])
    assert created_option.id == 2

    routes_tag_option_delete(1, api_config_dict["read_write"])

    with pytest.raises(HTTPException) as exc:
        routes_tag_option_get(1, api_config_dict["read"])
    assert exc.value.status_code == 404

    _ = routes_tag_option_get(2, api_config_dict["read"])

    options = routes_tag_option_list(post_tag.id, api_config_dict["read"])
    assert len(options) == 1

    assert options[0].id == 2
    assert options[0].order == sample_options[1].order
    assert options[0].value == sample_options[1].value
    assert options[0].name == sample_options[1].name
    assert options[0].display_name == sample_options[1].name

#####################################################################

def test_list_unauthorized(post_tag, api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_list(post_tag.id, api_config_unauthorized)
    assert exc.value.status_code == 401


def test_read_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_get(1, api_config_unauthorized)
    assert exc.value.status_code == 401


def test_create_unauthorized(api_config_unauthorized, post_tag, sample_options):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_post(post_tag.id, sample_options[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_update_unauthorized(api_config_unauthorized, sample_options):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_put(1, sample_options[0], api_config_unauthorized)
    assert exc.value.status_code == 401


def test_delete_unauthorized(api_config_unauthorized):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_delete(1, api_config_unauthorized)
    assert exc.value.status_code == 401

#####################################################################

def test_create_no_rights(api_config_read, post_tag, sample_options):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_post(post_tag.id, sample_options[0], api_config_read)
    assert exc.value.status_code == 401


def test_update_no_rights(api_config_read, sample_options):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_put(1, sample_options[0], api_config_read)
    assert exc.value.status_code == 401


def test_delete_no_rights(api_config_read):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_delete(1, api_config_read)
    assert exc.value.status_code == 401

#####################################################################

@pytest.fixture
def wrong_owner(api_config_dict, post_tag, sample_options):
    _ = routes_tag_option_post(post_tag.id, sample_options[0], api_config_dict["read_write"])
    return api_config_dict["read_write_2"]


def test_read_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_get(1, wrong_owner)
    assert exc.value.status_code == 404


def test_update_wrong_owner(wrong_owner, sample_options):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_put(1, sample_options[0], wrong_owner)
    assert exc.value.status_code == 404


def test_delete_wrong_owner(wrong_owner):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_delete(1, wrong_owner)
    assert exc.value.status_code == 404

#####################################################################

@pytest.fixture
def no_tag(api_config_dict, post_tag, sample_options):
    routes_tag_delete(post_tag.id, api_config_dict["read_write"])
    return post_tag


def test_list_no_tag(no_tag, api_config_read):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_list(no_tag.id, api_config_read)
    assert exc.value.status_code == 404


def test_create_no_tag(no_tag, sample_options, api_config_dict):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_post(no_tag.id, sample_options[0], api_config_dict["read_write"])
    assert exc.value.status_code == 404

#####################################################################

def test_list_wrong_tag_owner(post_tag, api_config_dict):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_list(post_tag.id, api_config_dict["read_2"])
    assert exc.value.status_code == 404


def test_create_wrong_tag_owner(post_tag, sample_options, api_config_dict):
    with pytest.raises(HTTPException) as exc:
        routes_tag_option_post(post_tag.id, sample_options[0], api_config_dict["read_write_2"])
    assert exc.value.status_code == 404
