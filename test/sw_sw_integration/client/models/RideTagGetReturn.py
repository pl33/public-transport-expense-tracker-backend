from typing import *

from pydantic import BaseModel, Field

from .RideTagLink import RideTagLink
from .Tag import Tag


class RideTagGetReturn(BaseModel):
    """
    None model

    """

    model_config = {"populate_by_name": True, "validate_assignment": True}

    link: RideTagLink = Field(validation_alias="link")

    tag: Tag = Field(validation_alias="tag")
